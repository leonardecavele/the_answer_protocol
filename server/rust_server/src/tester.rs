use rand::random_range;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tracing::warn;

const BWRAP: &str = "/usr/bin/bwrap";
const CLANG: &str = "/usr/bin/clang";
const TESTS_PATH: &str = "assets/tests";

const MAX_CODE_SIZE: usize = 64 * 1024;
const MAX_FILE_NAME_SIZE: usize = 80;
const MAX_CONCURRENT_SANDBOXES: usize = 2;

const COMPILATION_TIMEOUT: Duration = Duration::from_secs(8);
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(5);

static TEMP_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);
static ACTIVE_SANDBOXES: AtomicUsize = AtomicUsize::new(0);

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

    let Some(_permit) = SandboxPermit::try_acquire() else {
        return false;
    };

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

fn sandboxed_test(code: &str, tests: &str) -> io::Result<bool> {
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

    // Bubblewrap reports a process killed by signal N as 128 + N. Keeping the
    // success code below 128 makes it impossible for a crash to pass by chance.
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

// The submitted object is linked without libc. This tiny trusted entry point
// is the only code allowed to issue the final exit syscall.
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

#[derive(Clone, Copy)]
struct ResourceLimits {
    address_space: libc::rlim_t,
    cpu_seconds: libc::rlim_t,
    file_size: libc::rlim_t,
    open_files: libc::rlim_t,
    stack_size: libc::rlim_t,
}

impl ResourceLimits {
    const fn compiler() -> Self {
        Self {
            address_space: 512 * 1024 * 1024,
            cpu_seconds: 5,
            file_size: 8 * 1024 * 1024,
            open_files: 128,
            stack_size: 32 * 1024 * 1024,
        }
    }

    const fn program() -> Self {
        Self {
            address_space: 256 * 1024 * 1024,
            cpu_seconds: 1,
            file_size: 0,
            open_files: 64,
            stack_size: 8 * 1024 * 1024,
        }
    }
}

fn run_limited(
    mut command: Command,
    timeout: Duration,
    limits: ResourceLimits,
    seccomp_fd: Option<i32>,
) -> io::Result<ExitStatus> {
    command
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // `pre_exec` may only perform async-signal-safe operations. setpgid,
    // setrlimit, dup2, and fcntl satisfy that constraint.
    unsafe {
        command.pre_exec(move || {
            if libc::setpgid(0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }

            set_limit(libc::RLIMIT_CORE, 0)?;
            set_limit(libc::RLIMIT_AS, limits.address_space)?;
            set_limit(libc::RLIMIT_CPU, limits.cpu_seconds)?;
            set_limit(libc::RLIMIT_FSIZE, limits.file_size)?;
            set_limit(libc::RLIMIT_NOFILE, limits.open_files)?;
            set_limit(libc::RLIMIT_STACK, limits.stack_size)?;

            if let Some(source_fd) = seccomp_fd {
                if libc::dup2(source_fd, 3) == -1 {
                    return Err(io::Error::last_os_error());
                }
                if libc::fcntl(3, libc::F_SETFD, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }

    let mut child = command.spawn()?;
    let started_at = Instant::now();

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started_at.elapsed() >= timeout {
            // The outer process is its own group. Killing both the group and
            // bubblewrap's PID namespace init ensures no descendant survives.
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            return child.wait();
        }
        thread::sleep(POLL_INTERVAL);
    }
}

unsafe fn set_limit(resource: libc::__rlimit_resource_t, value: libc::rlim_t) -> io::Result<()> {
    let limit = libc::rlimit {
        rlim_cur: value,
        rlim_max: value,
    };
    if unsafe { libc::setrlimit(resource, &limit) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn write_private_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

struct SandboxDirectory {
    path: PathBuf,
}

impl SandboxDirectory {
    fn create() -> io::Result<Self> {
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

struct SandboxPermit;

impl SandboxPermit {
    fn try_acquire() -> Option<Self> {
        ACTIVE_SANDBOXES
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_CONCURRENT_SANDBOXES).then_some(active + 1)
            })
            .ok()
            .map(|_| Self)
    }
}

impl Drop for SandboxPermit {
    fn drop(&mut self) {
        ACTIVE_SANDBOXES.fetch_sub(1, Ordering::Release);
    }
}

// Bubblewrap consumes a classic BPF seccomp program as raw `sock_filter`
// records. The runtime uses an allowlist: no filesystem access, networking,
// process creation, ptrace, namespaces, BPF, or other kernel-facing APIs are
// available to submitted code.
fn create_seccomp_filter(directory: &Path) -> io::Result<File> {
    let path = directory.join("seccomp.bpf");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;

    for instruction in seccomp_instructions()? {
        file.write_all(&instruction.code.to_ne_bytes())?;
        file.write_all(&[instruction.jump_true, instruction.jump_false])?;
        file.write_all(&instruction.value.to_ne_bytes())?;
    }
    file.flush()?;
    file.rewind()?;
    Ok(file)
}

#[derive(Clone, Copy)]
struct BpfInstruction {
    code: u16,
    jump_true: u8,
    jump_false: u8,
    value: u32,
}

fn seccomp_instructions() -> io::Result<Vec<BpfInstruction>> {
    const BPF_LOAD_WORD_ABSOLUTE: u16 = 0x20;
    const BPF_JUMP_EQUAL: u16 = 0x15;
    const BPF_RETURN: u16 = 0x06;
    const SECCOMP_ALLOW: u32 = 0x7fff_0000;
    const SECCOMP_KILL_PROCESS: u32 = 0x8000_0000;

    #[cfg(target_arch = "x86_64")]
    const AUDIT_ARCH: u32 = 0xc000_003e;
    #[cfg(target_arch = "aarch64")]
    const AUDIT_ARCH: u32 = 0xc000_00b7;
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the seccomp filter only supports x86_64 and aarch64",
    ));

    let mut instructions = vec![
        // seccomp_data.arch lives at offset 4.
        BpfInstruction {
            code: BPF_LOAD_WORD_ABSOLUTE,
            jump_true: 0,
            jump_false: 0,
            value: 4,
        },
        BpfInstruction {
            code: BPF_JUMP_EQUAL,
            jump_true: 1,
            jump_false: 0,
            value: AUDIT_ARCH,
        },
        BpfInstruction {
            code: BPF_RETURN,
            jump_true: 0,
            jump_false: 0,
            value: SECCOMP_KILL_PROCESS,
        },
        // seccomp_data.nr lives at offset 0.
        BpfInstruction {
            code: BPF_LOAD_WORD_ABSOLUTE,
            jump_true: 0,
            jump_false: 0,
            value: 0,
        },
    ];

    let allowed_syscalls = [
        // Bubblewrap installs the same filter in its PID-namespace init. That
        // tiny reaper needs write/close/wait4, while the submitted freestanding
        // program only needs the initial execve and the trusted final exit.
        libc::SYS_write,
        libc::SYS_close,
        libc::SYS_wait4,
        libc::SYS_rt_sigreturn,
        libc::SYS_execve,
        libc::SYS_exit,
        libc::SYS_exit_group,
    ];

    for syscall in allowed_syscalls {
        instructions.push(BpfInstruction {
            code: BPF_JUMP_EQUAL,
            jump_true: 0,
            jump_false: 1,
            value: syscall as u32,
        });
        instructions.push(BpfInstruction {
            code: BPF_RETURN,
            jump_true: 0,
            jump_false: 0,
            value: SECCOMP_ALLOW,
        });
    }

    instructions.push(BpfInstruction {
        code: BPF_RETURN,
        jump_true: 0,
        jump_false: 0,
        value: SECCOMP_KILL_PROCESS,
    });
    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANSWER_TESTS: &str = r#"
        extern int answer(int);
        int main(void) {
            if (answer(0) != 42 || answer(-5) != 37) return 0;
            return SANDBOX_SUCCESS;
        }
    "#;

    #[test]
    fn rejects_unsafe_file_names() {
        assert!(!valid_file_name("../answer.c"));
        assert!(!valid_file_name("answer.c/other.c"));
        assert!(!valid_file_name("answer.cpp"));
        assert!(valid_file_name("answer.c"));
    }

    #[test]
    fn loads_tests_from_the_tests_directory() {
        assert!(!tests_for("add_42.c").unwrap().is_empty());
    }

    #[test]
    fn reports_a_missing_test_file() {
        let error = tests_for("missing_exercise.c").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    // These tests require Linux user namespaces, bubblewrap, and a static
    // clang toolchain. Run explicitly with:
    // cargo test tester::tests::sandbox -- --ignored
    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_accepts_correct_code() {
        assert!(sandboxed_test("int answer(int n) { return n + 42; }", ANSWER_TESTS).unwrap());
    }

    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_public_api_uses_the_requested_exercise() {
        assert!(test("add_one.c", "int add_one(int n) { return n + 1; }"));
        assert!(!test("add_one.c", "int add_one(int n) { return n; }"));
    }

    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_contains_crashes() {
        let code = "int answer(int n) { int *p = (int *)0; *p = n; return 42; }";
        assert!(!sandboxed_test(code, ANSWER_TESTS).unwrap());
    }

    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_rejects_inline_assembly() {
        let code = "int answer(int n) { __asm__(\"nop\"); return n + 42; }";
        assert!(!sandboxed_test(code, ANSWER_TESTS).unwrap());
    }

    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_stops_infinite_loops() {
        let code = "int answer(int n) { (void)n; for (;;) {} }";
        assert!(!sandboxed_test(code, ANSWER_TESTS).unwrap());
    }

    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_blocks_process_creation() {
        let code = r#"
            extern int fork(void);
            int answer(int n) {
                int child = fork();
                return child == -1 ? n + 42 : 0;
            }
        "#;
        assert!(!sandboxed_test(code, ANSWER_TESTS).unwrap());
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    #[ignore = "requires bubblewrap and clang"]
    fn sandbox_blocks_a_raw_syscall_hidden_in_executable_bytes() {
        let code = r#"
            __attribute__((section(".text")))
            static const unsigned char raw_fork[] = {
                0xb8, 0x39, 0x00, 0x00, 0x00, /* mov $57, %eax */
                0x0f, 0x05,                   /* syscall */
                0xc3                          /* ret */
            };
            int answer(int n) {
                int (*run)(void) = (int (*)(void))raw_fork;
                (void)run();
                return n + 42;
            }
        "#;
        assert!(!sandboxed_test(code, ANSWER_TESTS).unwrap());
    }
}
