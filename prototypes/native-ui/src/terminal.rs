//! Independently written PTY/grid/input fixture, not Zed's GPL terminal crates.
//! The child is a bounded synthetic program, never a shell/agent launcher.
use crate::{
    isolation::GuardedRun,
    panes::{PaneId, Panes, ViewLease},
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};
use std::{
    io::{Read, Write},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

pub const MAX_ROWS: u16 = 48;
pub const MAX_COLS: u16 = 160;
pub const MAX_OUTPUT: usize = 8 * 1024 * 1024;
const MAX_INPUT: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalError {
    Guard,
    Start,
    Io,
    Bounds,
    StaleView,
    Stopped,
    Flood,
}
#[derive(Clone, Debug)]
pub struct Cell {
    pub text: String,
    pub fg: vt100::Color,
    pub bg: vt100::Color,
    pub wide: bool,
    pub continuation: bool,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub rows: u16,
    pub cols: u16,
    pub cells: Vec<Cell>,
    pub cursor: (u16, u16),
    pub alternate: bool,
    pub bracketed_paste: bool,
    pub sgr_mouse: bool,
    pub eof: bool,
    pub error: Option<TerminalError>,
}

pub struct Grid {
    parser: vt100::Parser,
    received: usize,
}
impl Grid {
    pub fn new(rows: u16, cols: u16) -> Result<Self, TerminalError> {
        bounds(rows, cols)?;
        Ok(Self {
            parser: vt100::Parser::new(rows, cols, 256),
            received: 0,
        })
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), TerminalError> {
        if bytes.len() > MAX_OUTPUT.saturating_sub(self.received) {
            return Err(TerminalError::Flood);
        }
        self.received += bytes.len();
        self.parser.process(bytes);
        Ok(())
    }
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), TerminalError> {
        bounds(rows, cols)?;
        self.parser.screen_mut().set_size(rows, cols);
        Ok(())
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.parser.screen();
        let (rows, cols) = s.size();
        let mut cells = Vec::with_capacity(usize::from(rows) * usize::from(cols));
        for row in 0..rows {
            for col in 0..cols {
                let c = s.cell(row, col).expect("within screen bounds");
                // Prevent pathological combining sequences from reaching the renderer.
                let text = c.contents().chars().take(64).collect();
                cells.push(Cell {
                    text,
                    fg: c.fgcolor(),
                    bg: c.bgcolor(),
                    wide: c.is_wide(),
                    continuation: c.is_wide_continuation(),
                });
            }
        }
        Snapshot {
            rows,
            cols,
            cells,
            cursor: s.cursor_position(),
            alternate: s.alternate_screen(),
            bracketed_paste: s.bracketed_paste(),
            sgr_mouse: s.mouse_protocol_mode() != vt100::MouseProtocolMode::None
                && s.mouse_protocol_encoding() == vt100::MouseProtocolEncoding::Sgr,
            eof: false,
            error: None,
        }
    }
}
fn bounds(rows: u16, cols: u16) -> Result<(), TerminalError> {
    if rows == 0 || cols == 0 || rows > MAX_ROWS || cols > MAX_COLS {
        Err(TerminalError::Bounds)
    } else {
        Ok(())
    }
}

/// Workspace owns this handle across every close/detach/reopen. Drop/app exit
/// stops and reaps only this child. No name/PID search or filesystem deletion.
pub struct Terminal {
    child: Option<Box<dyn Child + Send + Sync>>,
    master: Box<dyn MasterPty + Send>,
    writer: Option<Box<dyn Write + Send>>,
    reader: Option<JoinHandle<()>>,
    cancelled: Arc<AtomicBool>,
    grid: Arc<Mutex<Grid>>,
    latest: Arc<Mutex<Arc<Snapshot>>>,
}
impl Terminal {
    /// Executable is an explicit absolute path to THIS probe build. The public
    /// CLI never exposes an arbitrary command line. Paths remain OsStr values.
    pub fn start(
        run: &GuardedRun,
        executable: &Path,
        mode: FixtureMode,
    ) -> Result<Self, TerminalError> {
        run.revalidate().map_err(|_| TerminalError::Guard)?;
        if !executable.is_absolute() {
            return Err(TerminalError::Start);
        }
        let pair = portable_pty::native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TerminalError::Start)?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|_| TerminalError::Start)?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|_| TerminalError::Start)?;
        let mut command = CommandBuilder::new(executable);
        command.env_clear();
        command.env("TERM", "xterm-256color");
        command.env("LANG", "C.UTF-8");
        // Required only by Windows process/runtime startup; never profile roots.
        #[cfg(windows)]
        for key in ["SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        command.cwd(run.root().join("data"));
        command.arg("--fixture-child");
        command.arg(mode.name());
        command.arg(run.root());
        command.arg("--nonce");
        command.arg(run.nonce_for_child());
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|_| TerminalError::Start)?;
        drop(pair.slave);
        let grid = Arc::new(Mutex::new(Grid::new(24, 80)?));
        let latest = Arc::new(Mutex::new(Arc::new(grid.lock().unwrap().snapshot())));
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut terminal = Self {
            child: Some(child),
            master: pair.master,
            writer: Some(writer),
            reader: None,
            cancelled,
            grid,
            latest,
        };
        let (grid, latest, cancelled) = (
            terminal.grid.clone(),
            terminal.latest.clone(),
            terminal.cancelled.clone(),
        );
        terminal.reader = Some(
            std::thread::Builder::new()
                .name("yurherdr-pty-reader".into())
                .spawn(move || read_grid(reader, grid, latest, cancelled))
                .map_err(|_| TerminalError::Start)?,
        );
        Ok(terminal)
    }
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.latest.lock().unwrap().clone()
    }
    pub fn resize(
        &mut self,
        panes: &Panes,
        view: ViewLease,
        rows: u16,
        cols: u16,
    ) -> Result<(), TerminalError> {
        if view.pane != PaneId::Terminal || !panes.accepts(view) {
            return Err(TerminalError::StaleView);
        }
        bounds(rows, cols)?;
        if self.child.is_none() {
            return Err(TerminalError::Stopped);
        }
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TerminalError::Io)?;
        let mut grid = self.grid.lock().unwrap();
        grid.resize(rows, cols)?;
        *self.latest.lock().unwrap() = Arc::new(grid.snapshot());
        Ok(())
    }
    pub fn input(
        &mut self,
        panes: &Panes,
        view: ViewLease,
        bytes: &[u8],
    ) -> Result<(), TerminalError> {
        if view.pane != PaneId::Terminal || !panes.accepts(view) {
            return Err(TerminalError::StaleView);
        }
        if bytes.len() > MAX_INPUT {
            return Err(TerminalError::Bounds);
        }
        self.writer
            .as_mut()
            .ok_or(TerminalError::Stopped)?
            .write_all(bytes)
            .map_err(|_| TerminalError::Io)
    }
    pub fn paste(
        &mut self,
        panes: &Panes,
        view: ViewLease,
        text: &str,
    ) -> Result<(), TerminalError> {
        if text.len() > MAX_INPUT - 12 || text.contains('\x1b') {
            return Err(TerminalError::Bounds);
        }
        let bytes = if self.snapshot().bracketed_paste {
            format!("\x1b[200~{text}\x1b[201~")
        } else {
            text.to_owned()
        };
        self.input(panes, view, bytes.as_bytes())
    }
    pub fn mouse(
        &mut self,
        panes: &Panes,
        view: ViewLease,
        row: u16,
        col: u16,
        pressed: bool,
    ) -> Result<(), TerminalError> {
        let s = self.snapshot();
        if !s.sgr_mouse || row >= s.rows || col >= s.cols {
            return Err(TerminalError::Bounds);
        }
        self.input(
            panes,
            view,
            format!(
                "\x1b[<0;{};{}{}",
                col + 1,
                row + 1,
                if pressed { 'M' } else { 'm' }
            )
            .as_bytes(),
        )
    }
    pub fn stop(&mut self) -> Result<(), TerminalError> {
        self.cancelled.store(true, Ordering::Release);
        self.writer.take();
        let result = if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Ok(Some(_)) => Ok(()),
                _ => {
                    let killed = child.kill();
                    let waited = child.wait();
                    if killed.is_err() && waited.is_err() {
                        Err(TerminalError::Io)
                    } else {
                        Ok(())
                    }
                }
            }
        } else {
            Ok(())
        };
        if let Some(reader) = self.reader.take() {
            reader.join().map_err(|_| TerminalError::Io)?;
        }
        result
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
fn read_grid(
    mut reader: Box<dyn Read + Send>,
    grid: Arc<Mutex<Grid>>,
    latest: Arc<Mutex<Arc<Snapshot>>>,
    cancelled: Arc<AtomicBool>,
) {
    let mut bytes = [0; 4096];
    loop {
        if cancelled.load(Ordering::Acquire) {
            break;
        }
        match reader.read(&mut bytes) {
            Ok(n) if n > 0 => {
                let mut g = grid.lock().unwrap();
                let result = g.feed(&bytes[..n]);
                let mut snapshot = g.snapshot();
                snapshot.error = result.err();
                *latest.lock().unwrap() = Arc::new(snapshot);
                if result.is_err() {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            _ => {
                let mut snapshot = grid.lock().unwrap().snapshot();
                snapshot.eof = true;
                *latest.lock().unwrap() = Arc::new(snapshot);
                break;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum FixtureMode {
    Interactive,
    Eof,
    Flood,
}
impl FixtureMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Eof => "eof",
            Self::Flood => "flood",
        }
    }
}

/// Child entry: guard must have passed again before this is called. No process
/// creation, network, credentials, shell expansion or inherited environment.
pub fn fixture_child(_run: GuardedRun, mode: FixtureMode) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        // SAFETY: termios is initialized by tcgetattr before use; fd 0 is the
        // owned PTY slave. Raw mode affects this synthetic child only.
        unsafe {
            let mut term = std::mem::MaybeUninit::<libc::termios>::uninit();
            if libc::tcgetattr(0, term.as_mut_ptr()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut term = term.assume_init();
            libc::cfmakeraw(&mut term);
            if libc::tcsetattr(0, libc::TCSANOW, &term) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
    let mut out = std::io::stdout().lock();
    out.write_all(b"\x1b[2J\x1b[H\x1b[32mYurherdr synthetic PTY\x1b[0m\r\n")?;
    for bytes in [b"\xe7".as_slice(), b"\x95", b"\x8c e\xcc", b"\x81\r\n"] {
        out.write_all(bytes)?;
        out.flush()?;
    }
    out.write_all(b"\x1b[?2004h\x1b[?1000h\x1b[?1006hready\r\n")?;
    out.flush()?;
    match mode {
        FixtureMode::Eof => return Ok(()),
        FixtureMode::Flood => {
            let block = [b'x'; 4096];
            for _ in 0..(MAX_OUTPUT / 4096 + 2) {
                out.write_all(&block)?;
            }
            return out.flush();
        }
        FixtureMode::Interactive => {}
    }
    let mut input = [0; 1024];
    loop {
        let n = std::io::stdin().read(&mut input)?;
        if n == 0 {
            return Ok(());
        }
        // Echo printable commits while making control sequences visible safely.
        if input[..n].contains(&3) {
            return Ok(());
        }
        out.write_all(b"\r\necho: ")?;
        out.write_all(&input[..n])?;
        out.flush()?;
    }
}
