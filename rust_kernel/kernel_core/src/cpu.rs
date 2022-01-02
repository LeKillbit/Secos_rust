//! Interface with the CPU and its registers

use crate::segmem::GdtPointer;
use crate::interrupts::IdtPointer;
use crate::{PERIPHERALS, println, print};
use crate::paging::pagemem::PhysAddr;
use core::arch::asm;

#[inline]
pub unsafe fn out8(addr : u16, val : u8) {
    asm!("out dx, al",
         in("dx") addr,
         in("al") val);
}

#[inline]
pub unsafe fn in8(addr : u16) -> u8 {
    let val : u8;
    asm!("in al, dx",
         in("dx") addr,
         out("al") val);
    val
}

#[inline]
pub fn halt() -> ! {
    println!("halted!");
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

#[inline]
pub fn get_gdt(pointer : &GdtPointer) {
    unsafe {
        asm!("sgdt [{}]", in(reg) pointer);
    }
}

#[inline]
pub fn set_gdt(pointer : &GdtPointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) pointer);
    }
}

/// Set the cs register. We use const generics here since the selector
/// must be known at compile time to generate correct asm
#[inline]
pub fn set_cs<const cs : u16>() {
    unsafe {
        // There seems to be a bug with labels in intel syntax assembly
        // so we need to use at&t syntax here...
        asm!("ljmp ${selector}, $1f; 1:", 
             selector = const cs,
             options(att_syntax)); 
    }
}

#[inline]
pub fn get_cs() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, cs", out(reg) val);
        val
    }
}

#[inline]
pub fn set_ss(ss : u16) {
    unsafe {
        asm!("mov ss, {}", in(reg) ss);
    }
}

#[inline]
pub fn get_ss() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, ss", out(reg) val);
        val
    }
}

#[inline]
pub fn set_ds(ds : u16) {
    unsafe {
        asm!("mov ds, {}", in(reg) ds);
    }
}

#[inline]
pub fn get_ds() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, ds", out(reg) val);
        val
    }
}

#[inline]
pub fn get_es() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, es", out(reg) val);
        val
    }
}

#[inline]
pub fn get_fs() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, fs", out(reg) val);
        val
    }
}

#[inline]
pub fn get_gs() -> u16 {
    unsafe {
        let val : u16;
        asm!("mov {}, gs", out(reg) val);
        val
    }
}

#[inline]
pub fn set_es(es : u16) {
    unsafe {
        asm!("mov es, {}", in(reg) es);
    }
}

#[inline]
pub fn set_fs(fs : u16) {
    unsafe {
        asm!("mov fs, {}", in(reg) fs);
    }
}

#[inline]
pub fn set_gs(gs : u16) {
    unsafe {
        asm!("mov gs, {}", in(reg) gs);
    }
}

#[inline]
pub fn set_idt(idt : &IdtPointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt);
    }
}

#[inline]
pub fn flush_tss() {
    unsafe {
        asm!("mov ax, {selector}
              ltr ax",
              selector = const 0x28 | 3);
    }
}

/// First arg : CS, Second arg : EIP
#[macro_export]
macro_rules! far_jump {
    ($x:expr, $y:expr) => {
        unsafe {
            asm!("ljmp ${segment}, ${}",
                 sym $y,
                 segment = const $x,
                 options(att_syntax));
        }
    }
}

#[inline]
pub fn get_esp() -> u32 {
    unsafe {
        let val : u32;
        asm!("mov {}, esp", out(reg) val);
        val
    }
}

#[inline]
pub fn set_esp(val : u32) {
    unsafe {
        asm!("mov esp, {}", in(reg) val);
    }
}

#[inline]
pub fn get_cr3() -> PhysAddr {
    unsafe {
        let val : u32;
        asm!("mov {}, cr3", out(reg) val);
        PhysAddr(val)
    }
}

#[inline]
pub fn get_cr2() -> u32 {
    unsafe {
        let val : u32;
        asm!("mov {}, cr2", out(reg) val);
        val
    }
}
