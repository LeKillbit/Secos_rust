//! A virtual memory manager

use super::pagemem::*;
use super::*;
use super::physmem::*;
use crate::cpu::get_cr3;

/// A virtual address space 
pub struct VirtMem {
    /// The page directory associated with this virtual address space
    pgd : PageDirectory,

    /// The virtual allocator bitmap associated with this virtual address
    /// space
    allocator_bitmap : &'static mut [u8; PAGE_SIZE],
}

impl core::fmt::Debug for VirtMem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtMem : ( pgd : {:#x} )", self.pgd.get_paddr().0)
    }
}

impl VirtMem {
    /// Create a new `VirtMem`
    pub fn new() -> Self {
        let pgd = PageDirectory::new();
        let bitmap = unsafe { PhysMem::alloc_phys_zeroed() };
        unsafe { pgd.map_raw(VirtAddr(KERNEL_VMEM_ALLOCATOR_BITMAP), 
                    bitmap.0 | PAGE_PRESENT | PAGE_WRITE); }
        let bitmap = unsafe { 
            core::mem::transmute::<u32, &mut [u8; PAGE_SIZE]>
                (KERNEL_VMEM_ALLOCATOR_BITMAP)
        };
        Self {
            pgd : pgd,
            allocator_bitmap : bitmap,
        }
    }
    
    /// Get current virtual address space from cr3 register
    pub fn get_current() -> Self {
        let pgd = PageDirectory::from_paddr(get_cr3());
        let bitmap = unsafe { 
            core::mem::transmute::<u32, &mut [u8; PAGE_SIZE]>
                (KERNEL_VMEM_ALLOCATOR_BITMAP)
        };
        Self {
            pgd : pgd,
            allocator_bitmap : bitmap,
        }
    }

    /// Get the physical address of the page directory, typically for setting
    /// cr3
    pub fn get_pgd_paddr(&self) -> PhysAddr {
        self.pgd.get_paddr()
    }

    /// Add a new virtual memory mapping to the virtual address space 
    pub fn map(&self, vaddr : VirtAddr, size : usize, write : bool, 
               user : bool) {
        self.pgd.map(vaddr, size, write, user);
    }

    /// Map a raw pte entry to `vaddr`
    pub fn map_raw(&self, vaddr : VirtAddr, raw : u32) {
        unsafe {
            self.pgd.map_raw(vaddr, raw);
        }
    }

    /// Dynamically alloc `npages` pages of virtual memory
    /// Returns the `VirtAddr` of the allocation
    pub fn alloc_virt_pages(&mut self, npages : usize, write : bool, user : bool) 
            -> VirtAddr {

        // Find a free window of size npages
        let alloc_index = self.allocator_bitmap.windows(npages)
            .position(|x| x.iter().all(|&y| y == 0))
            .expect("Couldn't find enough free contiguous virtual pages");

        // Update allocator bitmap
        self.allocator_bitmap[alloc_index..alloc_index + npages]
            .iter_mut()
            .for_each(|x| *x = 1);

        // Determine allocation address
        let alloc_addr = VirtAddr(KERNEL_VMEM_BASE + 
                                  ((alloc_index * PAGE_SIZE) as u32)); 
        
        // Create the mapping in virtual memory
        self.map(alloc_addr, npages * PAGE_SIZE, write, user);

        alloc_addr
    }

    /// Free `npages` pages of memory at `addr`
    pub fn free_virt_pages(&mut self, addr : VirtAddr, npages : usize) {
        // Get the allocator bitmap index
        let bitmap_index = (addr.0 - KERNEL_VMEM_BASE) / (PAGE_SIZE as u32);
        let bitmap_index = bitmap_index as usize;

        // Check that pages in this region are allocated
        let pages_allocated = self.allocator_bitmap[bitmap_index..bitmap_index + npages]
                .iter()
                .position(|x| *x==0)
                .is_none();

        // Free backing physical memory
        let start_mapping = addr.0;
        let end_mapping = addr.0 + ((npages * PAGE_SIZE) as u32);
        for virt_page in (start_mapping..end_mapping).step_by(PAGE_SIZE) {
            let mapping = self.pgd.translate(VirtAddr(virt_page));
            //println!("Mapping : {:#x?}", mapping);
            if mapping.page.is_none() {
                panic!("Trying to free invalid physical memory");
            }
            unsafe { PhysMem::free_phys(mapping.page.unwrap()); }
        }

        // Update allocator bitmap
        self.allocator_bitmap[bitmap_index..bitmap_index + npages]
            .iter_mut()
            .for_each(|x| *x = 0);
    }
}
