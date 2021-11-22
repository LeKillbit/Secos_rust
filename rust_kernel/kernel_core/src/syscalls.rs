//! All syscall handlers

use crate::interrupts::InterruptContext;
use crate::{println, print, PERIPHERALS};
use crate::virtmem::*;
use crate::pagemem::*;
use crate::physmem::*;

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
        // Print_number syscall
        3 => {
            sys_print_number(ctx.regs.ecx);
        }
        // Mmap_shared syscall
        10 => {
            sys_mmap_shared(VirtAddr(ctx.regs.ecx), ctx.regs.edx as usize);
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

/// Print `num`
fn sys_print_number(num : u32) {
    println!("{}", num);
}

/// Map a shared memory region identified by `id` at `vaddr`
fn sys_mmap_shared(vaddr : VirtAddr, id : usize) {
    const MAX_SHARED_MAPPINGS : usize = 10;
    static mut MAPPINGS : [Option<PhysAddr>; MAX_SHARED_MAPPINGS] = 
        [None; MAX_SHARED_MAPPINGS];
    
    if id < 0 || id > MAX_SHARED_MAPPINGS {
        panic!("invalid shared mapping id : {}", id);
    }

    unsafe {
        if MAPPINGS[id].is_none() {
            let page = PhysMem::alloc_phys_zeroed();
            MAPPINGS[id] = Some(page);
        }

        let vspace = VirtMem::get_current();
        vspace.map_raw(vaddr, MAPPINGS[id].unwrap().0
                       | PAGE_PRESENT | PAGE_USER | PAGE_WRITE);

        println!("Mapped phys page {:#x} at {:#x}", MAPPINGS[id].unwrap().0,
            vaddr.0);
    }
}
