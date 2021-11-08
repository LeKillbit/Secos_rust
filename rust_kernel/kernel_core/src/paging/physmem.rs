//! Interactions with physical memory
//! Bitmap-based physical page allocator

use super::pagemem::{PhysAddr, VirtAddr, PAGE_SIZE};
use super::*;

/// Size calculation : (0x7fe0000 - 0x400000) / 4096
/// (MAX_USABLE_ADDR - BASE_ALLOCATOR) / PAGE_SIZE
const BITMAP_SIZE : usize = 0x7be0;

/// A 0 represent a free page, a 1 represent a used page
static mut ALLOCATOR_BITMAP : [u8; BITMAP_SIZE] = [0; BITMAP_SIZE];

/// The base address of the allocator area
const BASE_ALLOCATOR : usize = 0x400_000;

/// Empty struct representing physical memory
pub struct PhysMem;

impl PhysMem {
    /// Allocate a page of physical memory. Returns the `PhysAddr` of 
    /// allocated page. Panics if no memory is available
    pub unsafe fn alloc_phys() -> PhysAddr {
        for (i, &page) in ALLOCATOR_BITMAP.iter().enumerate() {
            if page == 0 {
                ALLOCATOR_BITMAP[i] = 1;
                return PhysAddr((BASE_ALLOCATOR + i * PAGE_SIZE) as u32);
            }
        }
        panic!("Out of memory");
    }

    /// Same as `alloc_page` but memory will be zeroed
    pub unsafe fn alloc_phys_zeroed() -> PhysAddr {
        let page = Self::alloc_phys();
        core::ptr::write_bytes(page.0 as *mut u8, 0, PAGE_SIZE);
        page
    }

    /// Free page of physical memory at `addr`
    pub unsafe fn free_phys(addr : PhysAddr) {
        if addr.0 & 0xfff != 0 {
            panic!("Freeing non-aligned address : {:#x}", addr.0);
        }

        let index = ((addr.0 - BASE_ALLOCATOR as u32) >> 12) as usize;
        if index > BITMAP_SIZE || addr.0 < BASE_ALLOCATOR as u32 {
            panic!("Freeing a page outside the bounds of the allocator : {:#x}",
                   addr.0); 
        }
        if ALLOCATOR_BITMAP[index] != 1 {
            panic!("Freeing non-allocated page : {:#x} at index {:#x}", 
                   addr.0, index);
        }

        ALLOCATOR_BITMAP[index] = 0;
    }

    /// Provides a virtual address for `size` bytes of physical memory at 
    /// `paddr`
    pub fn translate(paddr : PhysAddr, size : usize) 
            -> *const u8 {
        // Make sure the requested data fits inside the window
        if paddr.0 + (size as u32) > KERNEL_PHYS_WINDOW_SIZE {
            panic!("Trying to translate paddr {:#x} but size ({:#x}) too big",
                paddr.0, size);
        }

        (paddr.0 + KERNEL_PHYS_WINDOW_BASE) as *const u8
    }
}

