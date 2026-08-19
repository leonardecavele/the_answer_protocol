use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) fn create_seccomp_filter(directory: &Path) -> io::Result<File> {
    let path = directory.join("seccomp.bpf");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;

    for instruction in seccomp_instructions()? {
        file.write_all(&(instruction.code as u16).to_ne_bytes())?;
        file.write_all(&[instruction.jump_true, instruction.jump_false])?;
        file.write_all(&instruction.value.to_ne_bytes())?;
    }
    file.flush()?;
    file.rewind()?;
    Ok(file)
}

#[derive(Clone, Copy)]
#[repr(u16)]
enum BpfOpcode {
    LoadWordAbsolute = 0x20,
    JumpEqual = 0x15,
    Return = 0x06,
}

#[repr(u32)]
enum BpfResult {
    Allow = 0x7fff_0000,
    KillProcess = 0x8000_0000,
}

#[derive(Clone, Copy)]
struct BpfInstruction {
    code: BpfOpcode,
    jump_true: u8,
    jump_false: u8,
    value: u32,
}

fn seccomp_instructions() -> io::Result<Vec<BpfInstruction>> {
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
        BpfInstruction {
            code: BpfOpcode::LoadWordAbsolute,
            jump_true: 0,
            jump_false: 0,
            value: 4,
        },
        BpfInstruction {
            code: BpfOpcode::JumpEqual,
            jump_true: 1,
            jump_false: 0,
            value: AUDIT_ARCH,
        },
        BpfInstruction {
            code: BpfOpcode::Return,
            jump_true: 0,
            jump_false: 0,
            value: BpfResult::KillProcess as u32,
        },
        BpfInstruction {
            code: BpfOpcode::LoadWordAbsolute,
            jump_true: 0,
            jump_false: 0,
            value: 0,
        },
    ];

    let allowed_syscalls = [
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
            code: BpfOpcode::JumpEqual,
            jump_true: 0,
            jump_false: 1,
            value: syscall as u32,
        });
        instructions.push(BpfInstruction {
            code: BpfOpcode::Return,
            jump_true: 0,
            jump_false: 0,
            value: BpfResult::Allow as u32,
        });
    }

    instructions.push(BpfInstruction {
        code: BpfOpcode::Return,
        jump_true: 0,
        jump_false: 0,
        value: BpfResult::KillProcess as u32,
    });
    Ok(instructions)
}
