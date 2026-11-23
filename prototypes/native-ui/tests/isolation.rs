#![cfg(unix)]
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use yurherdr_native_ui_probe::isolation::{GuardError, prepare, validate};
static NEXT: AtomicU64 = AtomicU64::new(1);

struct Layout {
    parent: PathBuf,
    root: PathBuf,
    nonce: String,
}
impl Layout {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = format!("{:08x}{n:08x}{:048x}", std::process::id(), 0);
        let parent =
            std::env::temp_dir().join(format!("yurherdr-tests-{}-{n}", std::process::id()));
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let root = prepare(&parent, &nonce).unwrap();
        Self {
            parent,
            root,
            nonce,
        }
    }
    fn check(&self) -> Result<(), GuardError> {
        validate(&self.root, &self.nonce, None).map(|_| ())
    }
}
impl Drop for Layout {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).unwrap();
    }
}
fn snapshot(path: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut out = vec![];
    for e in fs::read_dir(path).unwrap() {
        let e = e.unwrap();
        let m = fs::symlink_metadata(e.path()).unwrap();
        out.push((e.path(), m.len(), m.modified().unwrap()));
        if m.is_dir() {
            out.extend(snapshot(&e.path()));
        }
    }
    out.sort();
    out
}
#[test]
fn guard_cli_is_read_only_and_native_entry_without_core_is_rejected() {
    let l = Layout::new();
    let before = snapshot(&l.root);
    let probe = env!("CARGO_BIN_EXE_yurherdr-native-ui-probe");
    let guard = std::process::Command::new(probe)
        .args([
            "--guard-only",
            l.root.to_str().unwrap(),
            "--nonce",
            &l.nonce,
        ])
        .env_remove("HC_SESSION")
        .output()
        .unwrap();
    assert!(guard.status.success());
    assert!(
        String::from_utf8(guard.stdout)
            .unwrap()
            .contains("runtime effects: zero")
    );
    let blocked = std::process::Command::new(probe)
        .args(["--native", l.root.to_str().unwrap(), "--nonce", &l.nonce])
        .env_remove("HC_SESSION")
        .output()
        .unwrap();
    assert!(!blocked.status.success());
    assert_eq!(before, snapshot(&l.root));
    assert!(prepare(&l.parent, &l.nonce).is_err());
    assert_eq!(before, snapshot(&l.root));
}
#[test]
fn inherited_session_including_empty_is_rejected_before_path_access() {
    for value in ["", "synthetic-live"] {
        assert_eq!(
            validate(Path::new("/does-not-exist"), "", Some(value.as_ref())).unwrap_err(),
            GuardError::InheritedSession
        );
    }
}
#[test]
fn live_profile_roots_are_rejected_without_creating_a_layout() {
    if let Some(home) = std::env::var_os("HOME") {
        let nonce = "a".repeat(64);
        let root = PathBuf::from(home).join(".config/herdr-app/yurherdr-run-aaaaaaaaaaaaaaaa");
        assert_eq!(
            validate(&root, &nonce, None).unwrap_err(),
            GuardError::InvalidPath
        );
    }
}
#[test]
fn native_profile_requires_every_owned_directory_without_changing_parent_environment() {
    let l = Layout::new();
    let run = validate(&l.root, &l.nonce, None).unwrap();
    assert_eq!(
        run.native_profile(|_| None),
        Err(GuardError::InvalidProfile)
    );
    let expected = |name: &str| {
        Some(
            l.root
                .join(match name {
                    "XDG_CONFIG_HOME" | "YURHERDR_CONFIG_DIR" => "config",
                    "XDG_DATA_HOME" | "XDG_STATE_HOME" | "YURHERDR_DATA_DIR" => "data",
                    "XDG_CACHE_HOME" | "YURHERDR_CACHE_DIR" => "cache",
                    _ => "logs",
                })
                .into_os_string(),
        )
    };
    assert_eq!(run.native_profile(expected), Ok(()));
    assert_eq!(
        run.native_profile(|name| if name == "YURHERDR_LOG_DIR" {
            Some(l.root.join("data").into_os_string())
        } else {
            expected(name)
        }),
        Err(GuardError::InvalidProfile)
    );
}
#[test]
fn aliases_traversal_and_insecure_permissions_are_rejected() {
    let l = Layout::new();
    assert!(validate(&l.root.join(".."), &l.nonce, None).is_err());
    assert!(validate(&l.root.join("."), &l.nonce, None).is_err());
    let alias = l.parent.join("alias");
    symlink(&l.root, &alias).unwrap();
    assert!(validate(&alias, &l.nonce, None).is_err());
    fs::remove_dir(l.root.join("cache")).unwrap();
    symlink(l.root.join("data"), l.root.join("cache")).unwrap();
    assert!(l.check().is_err());
    fs::remove_file(l.root.join("cache")).unwrap();
    fs::create_dir(l.root.join("cache")).unwrap();
    fs::set_permissions(l.root.join("cache"), fs::Permissions::from_mode(0o777)).unwrap();
    assert!(l.check().is_err());
}
#[test]
fn malformed_oversized_foreign_and_colliding_manifests_fail_closed() {
    let l = Layout::new();
    let manifest = l.root.join("run.json");
    let original = fs::read(&manifest).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    for replacement in [
        b"{".to_vec(),
        vec![b' '; 4097],
        {
            let mut v = value.clone();
            v["extra"] = true.into();
            serde_json::to_vec(&v).unwrap()
        },
        {
            let mut v = value.clone();
            v["directories"][0] = "../config".into();
            serde_json::to_vec(&v).unwrap()
        },
        {
            let mut v = value.clone();
            v["directories"][0] = "data".into();
            serde_json::to_vec(&v).unwrap()
        },
        {
            let mut v = value.clone();
            v["application"] = "other.app".into();
            serde_json::to_vec(&v).unwrap()
        },
    ] {
        fs::write(&manifest, replacement).unwrap();
        assert!(l.check().is_err());
    }
    fs::write(&manifest, original).unwrap();
    fs::hard_link(&manifest, l.root.join("alias.json")).unwrap();
    assert!(l.check().is_err());
}
#[test]
fn wrong_nonce_symlink_marker_and_unsafe_file_owner_mode_are_rejected() {
    let l = Layout::new();
    let marker = l.root.join(".yurherdr-run");
    fs::write(&marker, "0".repeat(64)).unwrap();
    assert_eq!(l.check().unwrap_err(), GuardError::MarkerMismatch);
    fs::remove_file(&marker).unwrap();
    symlink(l.root.join("run.json"), &marker).unwrap();
    assert!(l.check().is_err());
    fs::remove_file(&marker).unwrap();
    fs::write(&marker, &l.nonce).unwrap();
    fs::set_permissions(&marker, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(l.check().unwrap_err(), GuardError::UnsafeOwnership);
}
