use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Clone, Copy)]
pub(super) struct ResourceLimits {
    address_space: libc::rlim_t,
    cpu_seconds: libc::rlim_t,
    file_size: libc::rlim_t,
    open_files: libc::rlim_t,
    stack_size: libc::rlim_t,
}

impl ResourceLimits {
    pub(super) const fn compiler() -> Self {
        Self {
            address_space: 512 * 1024 * 1024,
            cpu_seconds: 5,
            file_size: 8 * 1024 * 1024,
            open_files: 128,
            stack_size: 32 * 1024 * 1024,
        }
    }

    pub(super) const fn program() -> Self {
        Self {
            address_space: 256 * 1024 * 1024,
            cpu_seconds: 1,
            file_size: 0,
            open_files: 64,
            stack_size: 8 * 1024 * 1024,
        }
    }
}

pub(super) fn run_limited(
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
