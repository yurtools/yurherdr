//! Narrow Linux evaluation harness for one pinned, independently built core.
//! Never invokes hc discovery/autostart, installed binaries or live spaces.
use crate::isolation::{GuardError, GuardedRun};

pub const CORE_REVISION: &str = "300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e";
pub const GHOSTTY_REVISION: &str = "1e5cdb865807abc990181efb8ebfa9122955cce2";

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use serde::Deserialize;
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        os::{
            fd::AsRawFd,
            unix::{
                fs::{MetadataExt, OpenOptionsExt},
                net::UnixStream,
                process::CommandExt,
            },
        },
        path::PathBuf,
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BuildSpec {
        source_revision: String,
        ghostty_revision: String,
        sha256: String,
        version: String,
    }
    pub struct DevelopmentCore {
        child: Option<Child>,
        root: PathBuf,
        socket: PathBuf,
        identity: String,
        session: String,
        log_reader: Option<std::thread::JoinHandle<()>>,
    }
    impl DevelopmentCore {
        pub fn start(run: &GuardedRun) -> Result<Self, GuardError> {
            run.revalidate()?;
            let spec: BuildSpec = serde_json::from_slice(&crate::isolation::small_file(
                &run.root().join("core/build.json"),
                4096,
            )?)
            .map_err(|_| GuardError::CoreNotVerified)?;
            if spec.source_revision != CORE_REVISION
                || spec.ghostty_revision != GHOSTTY_REVISION
                || spec.version != "0.1.22"
                || spec.sha256.len() != 64
            {
                return Err(GuardError::CoreNotVerified);
            }
            // A fresh run may contain only its staged executable and receipt;
            // no saved agents, machines, account/config imports or endpoints.
            for name in ["config", "data", "cache", "logs"] {
                if fs::read_dir(run.root().join(name))
                    .map_err(|_| GuardError::Io)?
                    .next()
                    .is_some()
                {
                    return Err(GuardError::CoreNotVerified);
                }
            }
            let entries = fs::read_dir(run.root().join("core"))
                .map_err(|_| GuardError::Io)?
                .take(3)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| GuardError::Io)?;
            if entries.len() != 2
                || entries
                    .iter()
                    .any(|e| e.file_name() != "build.json" && e.file_name() != "herdr-core")
            {
                return Err(GuardError::CoreNotVerified);
            }
            let executable = run.root().join("core/herdr-core");
            // A descriptor pins the bytes while hashing. Runtime mutation by a
            // malicious same-user actor is not a sandbox guarantee.
            let mut file = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(&executable)
                .map_err(|_| GuardError::CoreNotVerified)?;
            let m = file.metadata().map_err(|_| GuardError::Io)?;
            if !m.is_file()
                || m.nlink() != 1
                || m.uid() != unsafe { libc::geteuid() }
                || m.mode() & 0o077 != 0
                || m.mode() & 0o100 == 0
                || m.len() > 512 * 1024 * 1024
            {
                return Err(GuardError::UnsafeOwnership);
            }
            let mut hash = Sha256::new();
            let mut bytes = [0; 65536];
            loop {
                let n = file.read(&mut bytes).map_err(|_| GuardError::Io)?;
                if n == 0 {
                    break;
                }
                hash.update(&bytes[..n]);
            }
            if format!("{:x}", hash.finalize()) != spec.sha256 {
                return Err(GuardError::CoreNotVerified);
            }
            let session = format!("yurherdr-{}", &run.nonce_for_child()[..16]);
            let socket = run.root().join("core/c.sock");
            // Upstream's longest holder endpoint adds 21 bytes to this path.
            if socket.as_os_str().len() + 21 > 107 {
                return Err(GuardError::InvalidPath);
            }
            let log = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(run.root().join("logs/core.log"))
                .map_err(|_| GuardError::Io)?;
            let mut command = Command::new(&executable);
            command
                .arg("serve")
                .env_clear()
                .env(
                    "HOME",
                    std::env::var_os("HOME").ok_or(GuardError::InvalidPath)?,
                )
                .env("LANG", "C.UTF-8")
                .env("HC_SESSION", &session)
                .env("HC_SOCKET", &socket)
                .env("XDG_CONFIG_HOME", run.root().join("config"))
                .env("XDG_STATE_HOME", run.root().join("data"))
                .env("XDG_RUNTIME_DIR", run.root().join("core"))
                // No PATH is needed: serve executes no fixture/agent commands.
                .current_dir(run.root().join("data"))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped());
            let parent = std::process::id();
            // SAFETY: the post-fork closure uses only async-signal-safe libc
            // operations; no locks or allocation. This child is ours alone.
            unsafe {
                command.pre_exec(move || {
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0
                        || libc::getppid() as u32 != parent
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let child = command.spawn().map_err(|_| GuardError::CoreNotVerified)?;
            let mut owned = Self {
                child: Some(child),
                root: run.root().into(),
                socket,
                identity: String::new(),
                session,
                log_reader: None,
            };
            let mut stderr = owned
                .child
                .as_mut()
                .unwrap()
                .stderr
                .take()
                .ok_or(GuardError::Io)?;
            owned.log_reader = Some(
                std::thread::Builder::new()
                    .name("yurherdr-core-log".into())
                    .spawn(move || {
                        let mut file = log;
                        let mut remaining = 64 * 1024;
                        let mut chunk = [0; 4096];
                        loop {
                            match stderr.read(&mut chunk) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => {
                                    let keep = n.min(remaining);
                                    if file.write_all(&chunk[..keep]).is_err() {
                                        break;
                                    }
                                    remaining -= keep;
                                }
                            }
                        }
                    })
                    .map_err(|_| GuardError::Io)?,
            );
            let until = Instant::now() + Duration::from_secs(5);
            while !owned.socket.exists() {
                if owned
                    .child
                    .as_mut()
                    .unwrap()
                    .try_wait()
                    .map_err(|_| GuardError::Io)?
                    .is_some()
                    || Instant::now() >= until
                {
                    return Err(GuardError::CoreNotVerified);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let hello = owned.call("hello", json!({"generation":1,"framed":false}))?;
            let status = owned.call("server.status", json!({}))?;
            let identity_path = owned
                .root
                .join("data/herdr-core")
                .join(&owned.session)
                .join("core.json");
            let metadata = fs::symlink_metadata(&identity_path).map_err(|_| GuardError::Io)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 4096 {
                return Err(GuardError::CoreNotVerified);
            }
            let mut identity_bytes = vec![];
            fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(identity_path)
                .map_err(|_| GuardError::Io)?
                .take(4097)
                .read_to_end(&mut identity_bytes)
                .map_err(|_| GuardError::Io)?;
            if identity_bytes.len() > 4096 {
                return Err(GuardError::CoreNotVerified);
            }
            let identity: Value =
                serde_json::from_slice(&identity_bytes).map_err(|_| GuardError::CoreNotVerified)?;
            let id = hello["core"]["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or(GuardError::CoreNotVerified)?;
            if hello["generation"] != 1
                || hello["version"] != "0.1.22"
                || status["version"] != "0.1.22"
                || status["pid"] != owned.child.as_ref().unwrap().id()
                || status["live"] != 0
                || status["finished"] != 0
                || status["core"]["id"] != id
                || identity["id"] != id
            {
                return Err(GuardError::CoreNotVerified);
            }
            owned.identity = id.into();
            Ok(owned)
        }
        fn call(&self, method: &str, params: Value) -> Result<Value, GuardError> {
            let mut stream =
                UnixStream::connect(&self.socket).map_err(|_| GuardError::CoreNotVerified)?;
            let mut cred = std::mem::MaybeUninit::<libc::ucred>::uninit();
            let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            // SAFETY: getsockopt writes at most the provided ucred-sized buffer.
            let ok = unsafe {
                libc::getsockopt(
                    stream.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_PEERCRED,
                    cred.as_mut_ptr().cast(),
                    &mut len,
                )
            };
            if ok != 0 || len as usize != std::mem::size_of::<libc::ucred>() {
                return Err(GuardError::CoreNotVerified);
            }
            // SAFETY: successful getsockopt initialized the full ucred above.
            let cred = unsafe { cred.assume_init() };
            if cred.pid as u32 != self.child.as_ref().ok_or(GuardError::CoreNotVerified)?.id()
                || cred.uid != unsafe { libc::geteuid() }
            {
                return Err(GuardError::CoreNotVerified);
            }
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .map_err(|_| GuardError::Io)?;
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .map_err(|_| GuardError::Io)?;
            let request = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
            stream
                .write_all(format!("{request}\n").as_bytes())
                .map_err(|_| GuardError::Io)?;
            let mut bytes = vec![];
            BufReader::new(stream.take(65537))
                .read_until(b'\n', &mut bytes)
                .map_err(|_| GuardError::Io)?;
            if bytes.len() > 65536 || bytes.last() != Some(&b'\n') {
                return Err(GuardError::CoreNotVerified);
            }
            let reply: Value =
                serde_json::from_slice(&bytes).map_err(|_| GuardError::CoreNotVerified)?;
            if reply["id"] != 1 || reply["jsonrpc"] != "2.0" || reply.get("error").is_some() {
                return Err(GuardError::CoreNotVerified);
            }
            reply
                .get("result")
                .cloned()
                .ok_or(GuardError::CoreNotVerified)
        }
        pub fn verify_for(&self, run: &GuardedRun) -> Result<(), GuardError> {
            run.revalidate()?;
            if run.root() != self.root || self.identity.is_empty() {
                return Err(GuardError::CoreNotVerified);
            }
            let status = self.call("server.status", json!({}))?;
            if status["core"]["id"] != self.identity {
                return Err(GuardError::CoreNotVerified);
            }
            Ok(())
        }
        pub fn stop(&mut self) -> Result<(), GuardError> {
            let result = self.stop_child();
            if let Some(reader) = self.log_reader.take() {
                reader.join().map_err(|_| GuardError::Io)?;
            }
            result
        }
        fn stop_child(&mut self) -> Result<(), GuardError> {
            if self.child.is_none() {
                return Ok(());
            }
            let _ = self.call("server.stop", json!({}));
            let mut child = self.child.take().unwrap();
            let until = Instant::now() + Duration::from_secs(3);
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return if status.success() {
                            Ok(())
                        } else {
                            Err(GuardError::CoreNotVerified)
                        };
                    }
                    Err(_) => break,
                    _ => {}
                }
                if Instant::now() >= until {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let _ = child.kill();
            child.wait().map_err(|_| GuardError::Io)?;
            Err(GuardError::CoreNotVerified)
        }
    }
    impl Drop for DevelopmentCore {
        fn drop(&mut self) {
            let _ = self.stop();
        }
    }
}
#[cfg(target_os = "linux")]
pub use linux::DevelopmentCore;

#[cfg(not(target_os = "linux"))]
pub struct DevelopmentCore;
#[cfg(not(target_os = "linux"))]
impl DevelopmentCore {
    pub fn start(_: &GuardedRun) -> Result<Self, GuardError> {
        Err(GuardError::PlatformNotAudited)
    }
    pub fn verify_for(&self, _: &GuardedRun) -> Result<(), GuardError> {
        Err(GuardError::PlatformNotAudited)
    }
    pub fn stop(&mut self) -> Result<(), GuardError> {
        Ok(())
    }
}
