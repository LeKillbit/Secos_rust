pub mod physmem;
pub mod pagemem;

/// The virtual base in the kernel page table where physical memory is 
/// linearly mapped. If set to 0, virtual memory is identity mapped to
/// physical memory
pub const KERNEL_PHYS_WINDOW_BASE : u32 = 0;

/// Size of the kernel physical window = 128 Mb (size of ram)
pub const KERNEL_PHYS_WINDOW_SIZE : u32 = 128 * 1024 * 1024;

use pagemem::*;
use crate::println;
use crate::print;
use crate::PERIPHERALS;

pub fn enable_paging() {
    unsafe {
        asm!("mov eax, cr0
              or eax, 0x80000000
              mov cr0, eax");
    }
}

/// Change the current cr3 value to point to a new Pag
pub fn switch_pgd(pgd : PageDirectory) {
    unsafe {
        asm!("mov eax, {}
              mov cr3, eax",
              in(reg) pgd.get_paddr().0);
    }
}

pub fn setup_identity_mapping() -> PageDirectory {
    let mut pageDirectory = PageDirectory::new();
    
    for paddr in (0..1024*1024*128).step_by(PAGE_SIZE) {
        let vaddr = VirtAddr(KERNEL_PHYS_WINDOW_BASE + paddr);
        unsafe {
            pageDirectory.map_raw(vaddr, paddr | PAGE_PRESENT | PAGE_WRITE);
        }
    }

    pageDirectory
}
