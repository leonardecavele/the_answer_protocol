use super::process::{ResourceLimits, run_limited};
use super::sandbox_directory::{SandboxDirectory, write_private_file};
use super::seccomp::create_seccomp_filter;
use rand::random_range;
use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const BWRAP: &str = "/usr/bin/bwrap";
const CLANG: &str = "/usr/bin/clang";

const COMPILATION_TIMEOUT: Duration = Duration::from_secs(8);
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) fn sandboxed_test(code: &str, tests: &str) -> io::Result<bool> {
    ensure_runtime_requirements()?;

    let directory = SandboxDirectory::create()?;
    write_private_file(&directory.path.join("submission.c"), code.as_bytes())?;

    let submission_status = run_limited(
        compiler_command(
            &directory.path,
            &[
                "-std=c17",
                "-O0",
                "-fno-gnu-inline-asm",
                "-fno-asm-blocks",
                "-fno-builtin",
                "-fno-common",
                "-fno-stack-protector",
                "-fno-asynchronous-unwind-tables",
                "-ffreestanding",
                "-pedantic-errors",
                "-c",
                "/work/submission.c",
                "-o",
                "/work/submission.o",
            ],
        ),
        COMPILATION_TIMEOUT,
        ResourceLimits::compiler(),
        None,
    )?;
    if !submission_status.success() {
        return Ok(false);
    }

    let success_code = i32::from(random_range(32_u8..=125));
    let success_define = format!("-DSANDBOX_SUCCESS={success_code}");
    write_private_file(&directory.path.join("tests.c"), tests.as_bytes())?;
    write_private_file(
        &directory.path.join("start.S"),
        trusted_start_source()?.as_bytes(),
    )?;
    let tests_status = run_limited(
        compiler_command(
            &directory.path,
            &[
                "-std=c17",
                "-O2",
                "-fno-builtin",
                "-fno-common",
                "-fno-stack-protector",
                "-fno-asynchronous-unwind-tables",
                "-ffreestanding",
                "-pedantic-errors",
                success_define.as_str(),
                "-Dmain=sandbox_main",
                "/work/tests.c",
                "/work/submission.o",
                "/work/start.S",
                "-nostdlib",
                "-static",
                "-Wl,-e,_start",
                "-Wl,-z,noexecstack",
                "-o",
                "/work/program",
            ],
        ),
        COMPILATION_TIMEOUT,
        ResourceLimits::compiler(),
        None,
    )?;
    if !tests_status.success() {
        return Ok(false);
    }

    fs::set_permissions(
        directory.path.join("program"),
        fs::Permissions::from_mode(0o500),
    )?;

    let seccomp_file = create_seccomp_filter(&directory.path)?;
    let execution_status = run_limited(
        execution_command(&directory.path),
        EXECUTION_TIMEOUT,
        ResourceLimits::program(),
        Some(seccomp_file.as_raw_fd()),
    )?;

    Ok(execution_status.code() == Some(success_code))
}

// Provides a minimal architecture-specific entry point that calls sandbox_main,
// then exits the process directly through the Linux syscall using sandbox_main's
// return value as the exit code. This avoids relying on libc or the standard C
// runtime and deliberately crashes if the exit syscall unexpectedly returns.
fn trusted_start_source() -> io::Result<&'static str> {
    #[cfg(target_arch = "x86_64")]
    return Ok(r#"
        .section .text
        .global _start
        .hidden _start
        .type _start,@function
        .extern sandbox_main
    _start:
        xorq %rbp, %rbp
        andq $-16, %rsp
        call sandbox_main
        movl %eax, %edi
        movl $60, %eax
        syscall
        ud2
        .size _start, .-_start
        .section .note.GNU-stack,"",@progbits
    "#);

    #[cfg(target_arch = "aarch64")]
    return Ok(r#"
        .section .text
        .global _start
        .hidden _start
        .type _start,%function
        .extern sandbox_main
    _start:
        bl sandbox_main
        mov x8, #93
        svc #0
        brk #0
        .size _start, .-_start
        .section .note.GNU-stack,"",@progbits
    "#);

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the trusted entry point only supports x86_64 and aarch64",
    ))
}

fn ensure_runtime_requirements() -> io::Result<()> {
    for executable in [BWRAP, CLANG] {
        let metadata = fs::metadata(executable).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("required sandbox executable {executable} is unavailable: {error}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(io::Error::other(format!(
                "required sandbox executable {executable} is not a file"
            )));
        }
    }
    Ok(())
}

fn compiler_command(work_directory: &Path, clang_arguments: &[&str]) -> Command {
    let mut command = Command::new(BWRAP);
    common_bwrap_arguments(&mut command);
    command
        .args(["--setenv", "PATH", "/usr/bin"])
        .args(["--setenv", "HOME", "/tmp"])
        .args(["--setenv", "TMPDIR", "/tmp"])
        .args(["--setenv", "LC_ALL", "C"])
        .args(["--ro-bind", "/usr", "/usr"])
        .args(["--ro-bind-try", "/lib", "/lib"])
        .args(["--ro-bind-try", "/lib64", "/lib64"])
        .arg("--bind")
        .arg(work_directory)
        .arg("/work")
        .args(["--size", "33554432", "--tmpfs", "/tmp"])
        .args(["--chdir", "/work", "--", CLANG])
        .args(clang_arguments);
    command
}

fn execution_command(work_directory: &Path) -> Command {
    let mut command = Command::new(BWRAP);
    common_bwrap_arguments(&mut command);
    command
        .arg("--ro-bind")
        .arg(work_directory)
        .arg("/work")
        .args(["--chdir", "/work", "--seccomp", "3", "--", "/work/program"]);
    command
}

fn common_bwrap_arguments(command: &mut Command) {
    command.args([
        "--unshare-all",
        "--unshare-user",
        "--disable-userns",
        "--die-with-parent",
        "--new-session",
        "--cap-drop",
        "ALL",
        "--clearenv",
    ]);
}
