use include_dir::{Dir, include_dir};
use std::borrow::Cow;
use std::path::PathBuf;

static EMBEDDED: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../assets");

#[derive(Clone)]
pub enum Assets {
    Embedded,
    Directory(PathBuf),
}

impl Assets {
    pub fn new(directory: Option<PathBuf>) -> Self {
        match directory {
            Some(root) => Self::Directory(root),
            None => Self::Embedded,
        }
    }

    pub fn read(&self, relative: &str) -> Option<Cow<'static, [u8]>> {
        match self {
            Self::Embedded => EMBEDDED
                .get_file(relative)
                .map(|file| Cow::Borrowed(file.contents())),
            Self::Directory(root) => std::fs::read(root.join(relative)).ok().map(Cow::Owned),
        }
    }
}
