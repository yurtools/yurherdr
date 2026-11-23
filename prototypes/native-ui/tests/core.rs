#![cfg(target_os = "linux")]
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use yurherdr_native_ui_probe::{
    core::{CORE_REVISION, DevelopmentCore, GHOSTTY_REVISION},
    isolation::{prepare, validate},
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
            std::env::temp_dir().join(format!("yurherdr-core-tests-{}-{n}", std::process::id()));
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let root = prepare(&parent, &nonce).unwrap();
        Self {
            parent,
            root,
            nonce,
        }
    }
    fn receipt(&self, sha: &str) {
        let p = self.root.join("core/build.json");
        fs::write(&p,serde_json::to_vec(&serde_json::json!({"source_revision":CORE_REVISION,"ghostty_revision":GHOSTTY_REVISION,"version":"0.1.22","sha256":sha})).unwrap()).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).unwrap();
    }
}
#[test]
fn unverified_binary_and_existing_endpoint_cannot_start_or_connect() {
    let r = Run::new();
    let run = validate(&r.root, &r.nonce, None).unwrap();
    assert!(DevelopmentCore::start(&run).is_err());
    let exe = r.root.join("core/herdr-core");
    fs::write(&exe, b"never execute this").unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o700)).unwrap();
    r.receipt(&"0".repeat(64));
    assert!(DevelopmentCore::start(&run).is_err());
    assert_eq!(fs::read_dir(r.root.join("logs")).unwrap().count(), 0);
    let listener = std::os::unix::net::UnixListener::bind(r.root.join("core/c.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    assert!(DevelopmentCore::start(&run).is_err());
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
#[test]
#[ignore = "requires an explicitly supplied, independently pinned core build"]
fn independently_built_core_has_owned_peer_identity_and_idempotent_teardown() {
    let source =
        std::env::var_os("YURHERDR_TEST_CORE").expect("set only to the task-owned pinned build");
    let bytes = fs::read(source).unwrap();
    let r = Run::new();
    let exe = r.root.join("core/herdr-core");
    fs::write(&exe, &bytes).unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o700)).unwrap();
    r.receipt(&format!("{:x}", Sha256::digest(&bytes)));
    let run = validate(&r.root, &r.nonce, None).unwrap();
    let mut core = DevelopmentCore::start(&run).unwrap();
    core.verify_for(&run).unwrap();
    let foreign = Run::new();
    let other = validate(&foreign.root, &foreign.nonce, None).unwrap();
    assert!(core.verify_for(&other).is_err());
    core.stop().unwrap();
    core.stop().unwrap();
    assert!(!r.root.join("core/c.sock").exists());
    // A used state cannot be silently reused on a new launch.
    assert!(DevelopmentCore::start(&run).is_err());
}

#[cfg(feature = "native")]
#[test]
#[ignore = "requires the explicitly supplied independently pinned core build"]
fn headless_startup_rejects_promptly_and_tears_down_without_wrapper_kill() {
    use std::{
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let bytes =
        fs::read(std::env::var_os("YURHERDR_TEST_CORE").expect("supply pinned core")).unwrap();
    let r = Run::new();
    let exe = r.root.join("core/herdr-core");
    fs::write(&exe, &bytes).unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o700)).unwrap();
    r.receipt(&format!("{:x}", Sha256::digest(&bytes)));
    let mut command = Command::new(env!("CARGO_BIN_EXE_yurherdr-native-ui-probe"));
    command
        .args(["--native", r.root.to_str().unwrap(), "--nonce", &r.nonce])
        .env_clear()
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("LANG", "C.UTF-8")
        .current_dir(r.root.join("data"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, dir) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "data"),
        ("YURHERDR_CONFIG_DIR", "config"),
        ("YURHERDR_DATA_DIR", "data"),
        ("YURHERDR_CACHE_DIR", "cache"),
        ("YURHERDR_LOG_DIR", "logs"),
    ] {
        command.env(key, r.root.join(dir));
    }
    let began = Instant::now();
    let mut child = command.spawn().unwrap();
    while child.try_wait().unwrap().is_none() && began.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(10));
    }
    if child.try_wait().unwrap().is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("headless rejection required external intervention after ten seconds");
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(began.elapsed() < Duration::from_secs(10));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("headless backend; OS window unavailable")
    );
    assert!(
        !r.root.join("core/c.sock").exists(),
        "owned core must stop before native process exits"
    );
}
