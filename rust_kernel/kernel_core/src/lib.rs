#![no_std]
#![feature(asm)]
#![feature(global_asm)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod pic;
mod cpu;
mod serial;
mod multiboot;
mod utils;
mod peripherals;
mod segmem;
mod interrupts;
mod tasks;
mod paging;
mod userland_tasks;
mod syscalls;

use core::panic::PanicInfo;

use crate::pic::*;
use crate::serial::*;
use crate::multiboot::*;
use crate::peripherals::Peripherals;
use crate::segmem::*;
use crate::interrupts::*;
use crate::paging::virtmem::*;
use crate::paging::*;
//use crate::userland_tasks::*;

#[no_mangle]
#[link_section=".mbh"]
static mbh : [u32; 3] = [
    MBH_MAGIC,
    MBH_FLAGS,
    0_u32.wrapping_sub(MBH_MAGIC + MBH_FLAGS),
];

#[panic_handler]
fn panic(_info : &PanicInfo) -> ! {
    println!("[PANIC] {}", _info);
    cpu::halt();
}

extern "C" { 
    static __kernel_start__ : usize; 
    static __kernel_end__ : usize; 
}

// A global struct to store references to peripherals
static mut PERIPHERALS : Peripherals = Peripherals {
    serial : None,
};

fn print_kernel_mmap(info : &MultibootInfo) {
    println!("kernel mem [{:#p} - {:#p}]",
             &__kernel_start__,
             &__kernel_end__);
    println!("MBI flags : {:#x}", info.flags);
    println!("mmap length : {:#x}", info.mmap_length);
    println!("mmap addr : {:#x}", info.mmap_addr);

    const MAX_MMAP_ENTRIES : usize = 10;

    let entries = info.mmap_length as usize / 
        core::mem::size_of::<multiboot_mmap_entry>();

    let memory_map = unsafe {
        core::mem::transmute::
            <*const u32, &[multiboot_mmap_entry; MAX_MMAP_ENTRIES]>
            (info.mmap_addr as *const u32)
    };

    for entry in &memory_map[..entries] {
        println!("{:<#10x} - {:<#11x} ({})", 
                 entry.addr, entry.addr + entry.len, entry.ty);
    }
}

fn userland() {
    unsafe {
        asm!("push 1
              push 2
              push 3
              mov eax, 1
              int 0x80
        ");
    }
}

/// First rust function called after asm bootstrap code
/// We use the fastcall convention to pass the mbi_ptr given by GRUB to 
/// rust_main as the first argument in the ecx register in asm code
#[no_mangle]
pub extern "fastcall" fn rust_main(mbi_ptr : &MultibootInfo) {

    // Init the serial port so we can use the print!() and println!() macros
    serial_init();
    
    //print_kernel_mmap(mbi_ptr);

    // Init the gdt with the following segments
    //  0x00 null
    //  0x08 kernel code segment
    //  0x10 kernel data segment
    //  0x18 user code segment
    //  0x20 user data segment
    //  0x28 TSS segment
    gdt_init();

    // Creates an IDT and initialize the idt register
    interrupts_init();

    // Remap IRQ[00-07] to IDT[0x20-0x27] and IRQ[08-15] to IDT[0x28-0x2f]
    Pic::remap(0x20, 0x28);

    // Create the kernel page directory, setup to identity map physical memory
    // for the first 128 MB
    let mut kernel_vspace = VirtMem::new();
    setup_identity_mapping(&kernel_vspace);

    // Set the cr3 register to use the previously created page directory
    switch_vspace(&kernel_vspace);

    // Enable paging
    enable_paging();

    tasks::Task::new(b"first_task", userland_tasks::task1);
    tasks::Task::new(b"second_task", userland_tasks::task2);

    tasks::schedule();

    loop {}
    
    //cpu::halt();
}
