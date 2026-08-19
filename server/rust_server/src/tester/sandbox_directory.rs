use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::warn;

static TEMP_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn write_private_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

pub(super) struct SandboxDirectory {
    pub(super) path: PathBuf,
}

impl SandboxDirectory {
    pub(super) fn create() -> io::Result<Self> {
        for _ in 0..32 {
            let counter = TEMP_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
            let random_part: u64 = rand::random();
            let path = PathBuf::from(format!(
                "/tmp/the-answer-protocol-{}-{counter}-{random_part:016x}",
                std::process::id()
            ));

            match fs::create_dir(&path) {
                Ok(()) => {
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique sandbox directory",
        ))
    }
}

impl Drop for SandboxDirectory {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            warn!(path = %self.path.display(), %error, "could not remove sandbox directory");
        }
    }
}
