#![cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use yurherdr_native_ui_probe::{
    isolation::{prepare, validate},
    panes::{PaneId, Panes, WindowId},
    terminal::{FixtureMode, Grid, MAX_OUTPUT, Terminal, TerminalError},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
struct Run {
    parent: PathBuf,
    root: PathBuf,
    nonce: String,
}
impl Run {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = format!("{:08x}{n:08x}{:048x}", std::process::id(), 0);
        let parent =
            std::env::temp_dir().join(format!("yurherdr-pty-tests-{}-{n}", std::process::id()));
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let root = prepare(&parent, &nonce).unwrap();
        Self {
            parent,
            root,
            nonce,
        }
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).unwrap();
    }
}
fn wait(mut condition: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(30);
    while !condition() {
        assert!(Instant::now() < until, "PTY condition timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn text(t: &Terminal) -> String {
    t.snapshot().cells.iter().map(|c| c.text.as_str()).collect()
}
#[test]
fn parser_handles_split_utf8_wide_combining_color_cursor_alternate_and_limits() {
    let mut g = Grid::new(4, 12).unwrap();
    for b in "界e\u{301}".as_bytes() {
        g.feed(&[*b]).unwrap();
    }
    let s = g.snapshot();
    assert_eq!(s.cells[0].text, "界");
    assert!(s.cells[0].wide);
    assert!(s.cells[1].continuation);
    assert_eq!(s.cells[2].text, "e\u{301}");
    g.feed(b"\x1b[2;1H\x1b[31mR\x1b[?2004h\x1b[?1000h\x1b[?1006h")
        .unwrap();
    let s = g.snapshot();
    assert_eq!(s.cells[12].text, "R");
    assert_eq!(s.cells[12].fg, vt100::Color::Idx(1));
    assert!(s.bracketed_paste && s.sgr_mouse);
    g.feed(b"\x1b[?1049hALT").unwrap();
    assert!(g.snapshot().alternate);
    g.feed(b"\x1b[?1049l").unwrap();
    assert_eq!(g.snapshot().cells[0].text, "界");
    assert_eq!(g.resize(0, 80), Err(TerminalError::Bounds));
    assert_eq!(g.feed(&vec![0; MAX_OUTPUT + 1]), Err(TerminalError::Flood));
    assert_eq!(g.snapshot().cells[0].text, "界");
}
#[test]
fn real_pty_survives_view_close_reopens_and_has_one_resize_owner() {
    let r = Run::new();
    let run = validate(&r.root, &r.nonce, None).unwrap();
    let exe = r.root.join("data/fixture path 界");
    fs::copy(env!("CARGO_BIN_EXE_yurherdr-native-ui-probe"), &exe).unwrap();
    let mut terminal = Terminal::start(&run, &exe, FixtureMode::Interactive).unwrap();
    wait(|| text(&terminal).contains("ready"));
    assert!(text(&terminal).contains("界"));
    let mut panes = Panes::default();
    let source = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
    terminal
        .input(&panes, source, "IME 界\r".as_bytes())
        .unwrap();
    wait(|| text(&terminal).contains("IME"));
    for i in 2..20 {
        let ticket = panes
            .reserve(panes.view(PaneId::Terminal).unwrap(), WindowId(i))
            .unwrap();
        let view = panes.ready(ticket).unwrap();
        assert_eq!(
            terminal.resize(&panes, source, 10, 40),
            Err(TerminalError::StaleView)
        );
        terminal.resize(&panes, view, 20, 60).unwrap();
        panes.close(view).unwrap();
        assert!(!terminal.snapshot().eof);
        let view = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
        terminal.paste(&panes, view, "draft").unwrap();
    }
    let view = panes.view(PaneId::Terminal).unwrap();
    terminal.mouse(&panes, view, 0, 0, true).unwrap();
    terminal.stop().unwrap();
    terminal.stop().unwrap();
    assert_eq!(
        terminal.input(&panes, view, b"x"),
        Err(TerminalError::Stopped)
    );
}
#[test]
fn failed_start_eof_flood_and_partial_teardown_are_bounded() {
    let r = Run::new();
    let run = validate(&r.root, &r.nonce, None).unwrap();
    assert!(matches!(
        Terminal::start(&run, &r.root.join("missing"), FixtureMode::Eof),
        Err(TerminalError::Start)
    ));
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_yurherdr-native-ui-probe"));
    let mut eof = Terminal::start(&run, exe, FixtureMode::Eof).unwrap();
    wait(|| eof.snapshot().eof);
    eof.stop().unwrap();
    eof.stop().unwrap();
    let mut flood = Terminal::start(&run, exe, FixtureMode::Flood).unwrap();
    wait(|| flood.snapshot().error == Some(TerminalError::Flood));
    flood.stop().unwrap();
}

#[test]
fn independent_stop_interrupts_a_real_blocked_pty_writer_and_reaps_it() {
    let r = Run::new();
    let run = validate(&r.root, &r.nonce, None).unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_yurherdr-native-ui-probe"));
    let mut terminal = Terminal::start(&run, exe, FixtureMode::Stalled).unwrap();
    wait(|| text(&terminal).contains("ready"));
    let stop = terminal.stop_handle();
    let (started, ready) = std::sync::mpsc::channel();
    let (done, result) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut panes = Panes::default();
        let view = panes.reopen(PaneId::Terminal, WindowId(1)).unwrap();
        started.send(()).unwrap();
        let block = [b'x'; 16384];
        let mut outcome = Ok(());
        for _ in 0..128 {
            outcome = terminal.input(&panes, view, &block);
            if outcome.is_err() {
                break;
            }
        }
        let teardown = terminal.stop();
        done.send((outcome, teardown)).unwrap();
    });
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(
        matches!(
            result.recv_timeout(Duration::from_millis(100)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ),
        "stalled child must actually block writes before Stop"
    );
    let began = Instant::now();
    stop.request().unwrap();
    let completed = result.recv_timeout(Duration::from_secs(2));
    let _ = stop.request();
    worker.join().unwrap();
    let (write, teardown) = completed
        .expect("independent Stop must interrupt write and finish teardown within two seconds");
    assert!(write.is_err());
    assert!(teardown.is_ok());
    assert!(began.elapsed() < Duration::from_secs(2));
    stop.request().unwrap();
}
