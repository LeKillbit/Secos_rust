//! All syscall handlers

use crate::interrupts::InterruptContext;
use crate::{print, PERIPHERALS};

/// Handle a syscall
pub fn handle_syscall(ctx : &InterruptContext) {
    match ctx.regs.eax {
        // Exit syscall
        1 => {
            sys_exit();
        },
        // Write syscall
        2 => {
            sys_write(ctx.regs.ecx as *const u8, ctx.regs.edx);
        }
        _ => panic!("Unimplemented syscall : {:#x}", ctx.regs.eax),
    }
}

/// Exit syscall
fn sys_exit() {
    panic!("exit syscall");
}

/// Write syscall
fn sys_write(buffer : *const u8, size : u32) {
    let buf = unsafe { core::slice::from_raw_parts(buffer, size as usize) };
    print!("{}", core::str::from_utf8(buf)
           .expect("couldn't translate to uft8"));
}

