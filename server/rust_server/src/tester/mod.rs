mod process;
mod sandbox;
mod sandbox_directory;
mod sandbox_permit;
mod seccomp;

use sandbox::sandboxed_test;
use sandbox_permit::SandboxPermit;
use std::fs;
use std::io;
use std::path::Path;
use tracing::warn;

const TESTS_PATH: &str = "../assets/tests";

const MAX_CODE_SIZE: usize = 64 * 1024;
const MAX_FILE_NAME_SIZE: usize = 80;

pub fn test(file_name: &str, code: &str) -> bool {
    if !valid_file_name(file_name) || code.len() > MAX_CODE_SIZE || code.contains('\0') {
        return false;
    }

    let tests = match tests_for(file_name) {
        Ok(tests) => tests,
        Err(error) => {
            warn!(file_name, %error, "could not load exercise tests");
            return false;
        }
    };

    let _permit = SandboxPermit::acquire();

    match sandboxed_test(code, &tests) {
        Ok(passed) => passed,
        Err(error) => {
            warn!(file_name, %error, "code sandbox failed");
            false
        }
    }
}

fn tests_for(file_name: &str) -> io::Result<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(TESTS_PATH)
        .join(file_name);

    fs::read_to_string(&path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not read {}: {error}", path.display()),
        )
    })
}

fn valid_file_name(file_name: &str) -> bool {
    if file_name.is_empty() || file_name.len() > MAX_FILE_NAME_SIZE {
        return false;
    }

    let path = Path::new(file_name);
    path.parent() == Some(Path::new(""))
        && path.extension().and_then(|extension| extension.to_str()) == Some("c")
        && file_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}