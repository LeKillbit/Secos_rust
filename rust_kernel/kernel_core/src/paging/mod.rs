pub mod physmem;
pub mod pagemem;
pub mod virtmem;

use pagemem::*;
use virtmem::*;

/// The virtual base in the kernel page table where physical memory is 
/// linearly mapped. If set to 0, virtual memory is identity mapped to
/// physical memory
pub const KERNEL_PHYS_WINDOW_BASE : u32 = 0;

/// Size of the kernel physical window = 128 Mb (size of ram)
pub const KERNEL_PHYS_WINDOW_SIZE : u32 = 128 * 1024 * 1024;

/// Base virtual address to use for dynamic allocations
pub const KERNEL_VMEM_BASE : u32 = 0x1337_0000;

/// Base virtual address where to store the virtual allocator bitmap
pub const KERNEL_VMEM_ALLOCATOR_BITMAP : u32 = 0xdead_0000;

/// The base address of the allocator area
pub const PHYS_ALLOCATOR_BASE : usize = 0x400_000;

pub fn enable_paging() {
    unsafe {
        asm!("mov eax, cr0
              or eax, 0x80000000
              mov cr0, eax");
    }
}

/// Switch virtual address space
pub fn switch_vspace(vmem : &VirtMem) {
    unsafe {
        asm!("mov eax, {}
              mov cr3, eax",
              in(reg) vmem.get_pgd_paddr().0);
    }
}

/// Identity map the physical memory at virtual address 
/// `KERNEL_PHYS_WINDOW_BASE` on `vmem` address space
pub fn setup_identity_mapping(vmem : &VirtMem) {
    for paddr in (0..1024*1024*128).step_by(PAGE_SIZE) {
        let vaddr = VirtAddr(KERNEL_PHYS_WINDOW_BASE + paddr);
        vmem.map_raw(vaddr, paddr | PAGE_PRESENT | PAGE_WRITE);
    }
}
