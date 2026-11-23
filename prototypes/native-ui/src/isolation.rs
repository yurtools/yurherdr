//! Read-only prelaunch validation. No core lookup, process launch or UI calls.
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub const APP_ID: &str = "dev.yurherdr.native-ui-probe";
pub const DIRECTORIES: [&str; 5] = ["config", "data", "cache", "logs", "core"];
const MAX_MANIFEST: u64 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardError {
    InheritedSession,
    InvalidPath,
    UnsafeOwnership,
    InvalidManifest,
    MarkerMismatch,
    Io,
    PlatformNotAudited,
    CoreNotVerified,
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InheritedSession => "inherited HC_SESSION; launch outside the live core",
            Self::InvalidPath => "run paths must be absolute, private and free of aliases",
            Self::UnsafeOwnership => "run files must belong only to this user",
            Self::InvalidManifest => "invalid or unsupported run manifest",
            Self::MarkerMismatch => "run marker does not match the explicit nonce",
            Self::Io => "cannot read the owned run layout",
            Self::PlatformNotAudited => "platform ownership/reparse validation is pending",
            Self::CoreNotVerified => {
                "native launch blocked: independent development core not verified"
            }
        })
    }
}
impl std::error::Error for GuardError {}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    application: String,
    nonce: String,
    directories: [String; 5],
}

/// Not a security sandbox. Same-user malicious mutation is outside this probe.
/// No native/runtime authority is granted by this read-only validation alone.
#[derive(Debug)]
pub struct GuardedRun {
    root: PathBuf,
    nonce: String,
}

impl GuardedRun {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn nonce_for_child(&self) -> &str {
        &self.nonce
    }

    pub fn revalidate(&self) -> Result<(), GuardError> {
        validate(
            &self.root,
            &self.nonce,
            std::env::var_os("HC_SESSION").as_deref(),
        )
        .map(|_| ())
    }

    /// Intentionally closed until a built core and its endpoint/identity handshake
    /// have been verified. A manifest field cannot bypass this gate.
    pub fn authorize_native(&self) -> Result<(), GuardError> {
        self.revalidate()?;
        Err(GuardError::CoreNotVerified)
    }
}

fn nonce_valid(nonce: &str) -> bool {
    nonce.len() == 64
        && nonce
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn absolute_clean(path: &Path) -> Result<(), GuardError> {
    if !path.is_absolute()
        || path.components().any(|c| {
            !matches!(
                c,
                Component::RootDir | Component::Normal(_) | Component::Prefix(_)
            )
        })
    {
        return Err(GuardError::InvalidPath);
    }
    // Path::components normalizes interior `.`. Reject those before normalization.
    if path
        .as_os_str()
        .to_str()
        .is_none_or(|s| s.split(['/', '\\']).any(|p| p == "." || p == ".."))
    {
        return Err(GuardError::InvalidPath);
    }
    let mut at = PathBuf::new();
    for part in path.components() {
        at.push(part);
        let m = fs::symlink_metadata(&at).map_err(|_| GuardError::Io)?;
        if m.file_type().is_symlink() || !m.is_dir() {
            return Err(GuardError::InvalidPath);
        }
        check_ancestor(&m)?;
    }
    if fs::canonicalize(path).map_err(|_| GuardError::Io)? != path {
        return Err(GuardError::InvalidPath);
    }
    Ok(())
}

#[cfg(unix)]
fn check_ancestor(m: &fs::Metadata) -> Result<(), GuardError> {
    use std::os::unix::fs::MetadataExt;
    // Sticky shared /tmp is permitted; ordinary writable ancestors are not.
    if m.mode() & 0o022 != 0 && m.mode() & 0o1000 == 0 {
        return Err(GuardError::UnsafeOwnership);
    }
    Ok(())
}
#[cfg(not(unix))]
fn check_ancestor(_: &fs::Metadata) -> Result<(), GuardError> {
    Err(GuardError::PlatformNotAudited)
}

#[cfg(unix)]
fn private(m: &fs::Metadata) -> Result<(), GuardError> {
    use std::os::unix::fs::MetadataExt;
    // SAFETY: geteuid takes no pointers and has no side effects.
    if m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
        return Err(GuardError::UnsafeOwnership);
    }
    if m.is_file() && m.nlink() != 1 {
        return Err(GuardError::InvalidPath);
    }
    Ok(())
}
#[cfg(not(unix))]
fn private(_: &fs::Metadata) -> Result<(), GuardError> {
    Err(GuardError::PlatformNotAudited)
}

fn small_file(path: &Path, limit: u64) -> Result<Vec<u8>, GuardError> {
    let m = fs::symlink_metadata(path).map_err(|_| GuardError::Io)?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > limit {
        return Err(GuardError::InvalidManifest);
    }
    private(&m)?;
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let f = options.open(path).map_err(|_| GuardError::Io)?;
    let opened = f.metadata().map_err(|_| GuardError::Io)?;
    private(&opened)?;
    if !opened.is_file() {
        return Err(GuardError::InvalidManifest);
    }
    let mut bytes = Vec::new();
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| GuardError::Io)?;
    if bytes.len() as u64 > limit {
        return Err(GuardError::InvalidManifest);
    }
    Ok(bytes)
}

/// `inherited_session` is captured by the caller, never unset globally.
/// Even an empty inherited HC_SESSION is rejected (upstream defaults it).
pub fn validate(
    root: &Path,
    nonce: &str,
    inherited_session: Option<&OsStr>,
) -> Result<GuardedRun, GuardError> {
    if inherited_session.is_some() {
        return Err(GuardError::InheritedSession);
    }
    if !nonce_valid(nonce) {
        return Err(GuardError::MarkerMismatch);
    }
    absolute_clean(root)?;
    if root.file_name() != Some(OsStr::new(&format!("yurherdr-run-{}", &nonce[..16]))) {
        return Err(GuardError::InvalidPath);
    }
    private(&fs::symlink_metadata(root).map_err(|_| GuardError::Io)?)?;
    if small_file(&root.join(".yurherdr-run"), 64)? != nonce.as_bytes() {
        return Err(GuardError::MarkerMismatch);
    }
    let manifest: Manifest =
        serde_json::from_slice(&small_file(&root.join("run.json"), MAX_MANIFEST)?)
            .map_err(|_| GuardError::InvalidManifest)?;
    if manifest.version != 1
        || manifest.application != APP_ID
        || manifest.nonce != nonce
        || manifest.directories != DIRECTORIES
    {
        return Err(GuardError::InvalidManifest);
    }
    for name in DIRECTORIES {
        let path = root.join(name);
        absolute_clean(&path)?;
        private(&fs::symlink_metadata(path).map_err(|_| GuardError::Io)?)?;
    }
    Ok(GuardedRun {
        root: root.to_owned(),
        nonce: nonce.to_owned(),
    })
}

/// Explicit provisioning, separate from the zero-write guard. Never overwrites.
/// On partial failure, leave the marked layout for inspection; never recursively
/// remove a path that another actor might have replaced.
pub fn prepare(parent: &Path, nonce: &str) -> Result<PathBuf, GuardError> {
    if std::env::var_os("HC_SESSION").is_some() {
        return Err(GuardError::InheritedSession);
    }
    if !nonce_valid(nonce) {
        return Err(GuardError::MarkerMismatch);
    }
    absolute_clean(parent)?;
    let root = parent.join(format!("yurherdr-run-{}", &nonce[..16]));
    private_dir(&root)?;
    write_new(&root.join(".yurherdr-run"), nonce.as_bytes())?;
    for name in DIRECTORIES {
        private_dir(&root.join(name))?;
    }
    let manifest = Manifest {
        version: 1,
        application: APP_ID.into(),
        nonce: nonce.into(),
        directories: DIRECTORIES.map(String::from),
    };
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|_| GuardError::InvalidManifest)?;
    write_new(&root.join("run.json"), &bytes)?;
    Ok(root)
}

fn private_dir(path: &Path) -> Result<(), GuardError> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    #[cfg(not(unix))]
    {
        return Err(GuardError::PlatformNotAudited);
    }
    b.create(path).map_err(|_| GuardError::Io)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), GuardError> {
    use std::io::Write;
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o.open(path).map_err(|_| GuardError::Io)?;
    f.write_all(bytes).map_err(|_| GuardError::Io)?;
    f.sync_all().map_err(|_| GuardError::Io)
}
