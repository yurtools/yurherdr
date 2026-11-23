//! Bounded Linux/Wayland image read. The small helper bounds a potentially
//! stalled compositor handshake and metadata allocation as well as pipe bytes.
//! This is resource/crash containment, not a security sandbox or plugin host.
use crate::{
    clipboard::{DraftImage, PasteError},
    isolation::GuardedRun,
};

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::clipboard::{MAX_ENCODED, decode};
    use std::{
        io::{Read, Write},
        os::fd::AsRawFd,
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };

    struct OwnedChild(Option<Child>);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                if !matches!(child.try_wait(), Ok(Some(_))) {
                    let _ = child.kill();
                }
                let _ = child.wait();
            }
        }
    }
    pub fn read_image(run: &GuardedRun) -> Result<DraftImage, PasteError> {
        run.revalidate().map_err(|_| PasteError::Closed)?;
        if std::env::var_os("WAYLAND_DISPLAY").is_none() {
            return Err(PasteError::NativeUnavailable);
        }
        let mut command =
            Command::new(std::env::current_exe().map_err(|_| PasteError::Unsupported)?);
        command.args([
            std::ffi::OsStr::new("--clipboard-child"),
            run.root().as_os_str(),
            std::ffi::OsStr::new("--nonce"),
            std::ffi::OsStr::new(run.nonce_for_child()),
        ]);
        command
            .env_clear()
            .env("YURHERDR_PARENT_PID", std::process::id().to_string())
            .current_dir(run.root().join("data"));
        // Native display discovery only. No agent/core session, account store,
        // shell, executable search path or caller's service environment.
        for key in ["WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "HOME"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut owned = OwnedChild(Some(command.spawn().map_err(|_| PasteError::Busy)?));
        let output = owned
            .0
            .as_mut()
            .unwrap()
            .stdout
            .take()
            .ok_or(PasteError::Busy)?;
        let bytes = read_pipe(output, MAX_ENCODED + 1, Duration::from_secs(3))?;
        let status = owned
            .0
            .as_mut()
            .unwrap()
            .wait()
            .map_err(|_| PasteError::Busy)?;
        if !status.success() {
            return Err(match status.code() {
                Some(20) => PasteError::Empty,
                Some(21) => PasteError::NativeUnavailable,
                Some(23) => PasteError::Unsupported,
                Some(25) => PasteError::Malformed,
                Some(24) => PasteError::Oversized,
                _ => PasteError::Busy,
            });
        }
        let (format, bytes) = bytes.split_first().ok_or(PasteError::Empty)?;
        decode(
            match format {
                1 => "image/png",
                2 => "image/jpeg",
                _ => return Err(PasteError::Malformed),
            },
            bytes,
        )
    }
    fn nonblocking(fd: i32) -> Result<(), PasteError> {
        // SAFETY: fcntl only updates the owned pipe descriptor's status flags.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(PasteError::Busy);
        }
        Ok(())
    }
    pub(crate) fn read_pipe(
        mut pipe: impl Read + AsRawFd,
        limit: usize,
        timeout: Duration,
    ) -> Result<Vec<u8>, PasteError> {
        nonblocking(pipe.as_raw_fd())?;
        let deadline = Instant::now() + timeout;
        let mut bytes = Vec::new();
        let mut chunk = [0; 8192];
        loop {
            if Instant::now() >= deadline {
                return Err(PasteError::Busy);
            }
            match pipe.read(&mut chunk) {
                Ok(0) => return Ok(bytes),
                Ok(n) => {
                    if n > limit.saturating_sub(bytes.len()) {
                        return Err(PasteError::Oversized);
                    }
                    bytes.extend_from_slice(&chunk[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return Err(PasteError::Busy),
            }
        }
    }
    pub fn helper(run: GuardedRun) -> i32 {
        let result = (|| {
            run.revalidate().map_err(|_| PasteError::Closed)?;
            let parent = std::env::var("YURHERDR_PARENT_PID")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .ok_or(PasteError::Closed)?;
            // SAFETY: these limits and parent-death signal affect this owned
            // helper alone. Guard validation has already completed.
            unsafe {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0
                    || libc::getppid() as u32 != parent
                {
                    return Err(PasteError::Closed);
                }
                let memory = libc::rlimit {
                    rlim_cur: 128 * 1024 * 1024,
                    rlim_max: 128 * 1024 * 1024,
                };
                let cpu = libc::rlimit {
                    rlim_cur: 2,
                    rlim_max: 3,
                };
                if libc::setrlimit(libc::RLIMIT_DATA, &memory) != 0
                    || libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0
                {
                    return Err(PasteError::Unsupported);
                }
            }
            use wl_clipboard_rs::paste::{ClipboardType, Error, MimeType, Seat, get_contents};
            for (mime, tag) in [("image/png", 1u8), ("image/jpeg", 2)] {
                match get_contents(
                    ClipboardType::Regular,
                    Seat::Unspecified,
                    MimeType::Specific(mime),
                ) {
                    Ok((pipe, actual)) => {
                        if actual != mime {
                            return Err(PasteError::Malformed);
                        }
                        let bytes = read_pipe(pipe, MAX_ENCODED, Duration::from_secs(2))?;
                        if bytes.is_empty() {
                            return Err(PasteError::Empty);
                        }
                        let mut out = std::io::stdout().lock();
                        out.write_all(&[tag]).map_err(|_| PasteError::Busy)?;
                        out.write_all(&bytes).map_err(|_| PasteError::Busy)?;
                        return Ok(());
                    }
                    Err(Error::NoMimeType) => continue,
                    Err(Error::ClipboardEmpty) => return Err(PasteError::Empty),
                    Err(
                        Error::NoSeats
                        | Error::MissingProtocol { .. }
                        | Error::WaylandConnection(_)
                        | Error::SocketOpenError(_),
                    ) => return Err(PasteError::NativeUnavailable),
                    Err(_) => return Err(PasteError::Busy),
                }
            }
            Err(PasteError::Unsupported)
        })();
        match result {
            Ok(()) => 0,
            Err(PasteError::Empty) => 20,
            Err(PasteError::NativeUnavailable) => 21,
            Err(PasteError::Unsupported) => 23,
            Err(PasteError::Malformed) => 25,
            Err(PasteError::Oversized) => 24,
            Err(_) => 22,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn native_pipe_eof_flood_and_stalled_owner_are_bounded() {
            use std::os::unix::net::UnixStream;
            let (read, mut write) = UnixStream::pair().unwrap();
            write.write_all(b"png").unwrap();
            drop(write);
            assert_eq!(read_pipe(read, 16, Duration::from_secs(1)).unwrap(), b"png");
            let (read, mut write) = UnixStream::pair().unwrap();
            write.write_all(&[0; 17]).unwrap();
            assert!(matches!(
                read_pipe(read, 16, Duration::from_secs(1)),
                Err(PasteError::Oversized)
            ));
            drop(write);
            let (read, _held_owner) = UnixStream::pair().unwrap();
            assert!(matches!(
                read_pipe(read, 16, Duration::from_millis(30)),
                Err(PasteError::Busy)
            ));
        }
    }
}
#[cfg(target_os = "linux")]
pub use linux::{helper, read_image};
#[cfg(not(target_os = "linux"))]
pub fn read_image(_: &GuardedRun) -> Result<DraftImage, PasteError> {
    Err(PasteError::NativeUnavailable)
}
#[cfg(not(target_os = "linux"))]
pub fn helper(_: GuardedRun) -> i32 {
    21
}
