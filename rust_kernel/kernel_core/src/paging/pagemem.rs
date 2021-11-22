//! Pagination structures and methods

use super::physmem::*;
use core::mem::size_of;

pub const PAGE_SIZE : usize = 0x1000;

/// Page table flag indicating the entry is valid
pub const PAGE_PRESENT: u32 = 1 << 0;

/// Page table flag indicating this page or table is writable
pub const PAGE_WRITE: u32 = 1 << 1;

/// Page table flag indicating this page or table is accessible in user mode
pub const PAGE_USER: u32 = 1 << 2;

/// Page table flag indicating this page or table has write-through caching
/// enabled
pub const PAGE_WRITE_THROUGH_ENABLE: u32 = 1 << 3;

/// Page table flag indiciating that accesses to the memory described by this
/// page or table should be strongly uncached
pub const PAGE_CACHE_DISABLE: u32 = 1 << 4;

/// Page has been accessed
pub const PAGE_ACCESSED: u32 = 1 << 5;

/// Page has been dirtied
pub const PAGE_DIRTY: u32 = 1 << 6;

/// Page table flag indicating that this page entry is a large page
pub const PAGE_LARGE: u32 = 1 << 7;

/// A strongly typed Virtual Address
#[derive(Debug, Copy, Clone)]
pub struct VirtAddr(pub u32);

/// A strongly typed Physical Address
#[derive(Debug, Copy, Clone)]
pub struct PhysAddr(pub u32);

/// State of a page table mapping
#[derive(Debug)]
pub struct Mapping {
    /// Physical address of the page directory entry for this mapping
    pub pde : Option<PhysAddr>,

    /// Physical address of the page table entry for this mapping
    pub pte : Option<PhysAddr>,

    /// Base address of the physical page backing the virtual memory
    pub page : Option<PhysAddr>,
}

/// A Page Directory Entry
pub struct PageDirectoryEntry(pub u32);

impl PageDirectoryEntry {
    pub fn new(val : u32) -> Self {
        Self(val)
    }

    pub fn set_present(&mut self, val : u32) {
        let val = val & 1;
        // Set the first bit of the pde to val
        self.0 = (self.0 & !(1 << 0)) | (val << 0);
    }

    pub fn set_rw(&mut self, val : u32) {
        let val = val & 1;
        // Set the second bit of the pde to val
        self.0 = (self.0 & !(1 << 1)) | (val << 1);
    }

    pub fn set_priv(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 2)) | (val << 2);
    }

    pub fn set_write_through(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 3)) | (val << 3);
    }

    pub fn set_cache_disable(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 4)) | (val << 4);
    }

    pub fn set_accessed(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 5)) | (val << 5);
    }

    pub fn set_pagesize(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 7)) | (val << 7);
    }

    pub fn set_paddr(&mut self, addr : PhysAddr) {
        self.0 = addr.0 | self.0 & 0xfff;
    }

    pub fn get_paddr(&self) -> PhysAddr {
        PhysAddr(self.0 & !0xfff)
    }
}

/// A Page Table Entry
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub fn new(val : u32) -> Self {
        Self(val)
    }

    pub fn set_present(&mut self, val : u32) {
        let val = val & 1;
        // Set the first bit of the pde to val
        self.0 = (self.0 & !(1 << 0)) | (val << 0);
    }

    pub fn set_rw(&mut self, val : u32) {
        let val = val & 1;
        // Set the second bit of the pde to val
        self.0 = (self.0 & !(1 << 1)) | (val << 1);
    }

    pub fn set_priv(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 2)) | (val << 2);
    }

    pub fn set_write_through(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 3)) | (val << 3);
    }

    pub fn set_cache_disable(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 4)) | (val << 4);
    }

    pub fn set_accessed(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 5)) | (val << 5);
    }

    pub fn set_dirty(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 6)) | (val << 6);
    }

    pub fn set_page_attribute_table(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 7)) | (val << 7);
    }

    pub fn set_global(&mut self, val : u32) {
        let val = val & 1;
        self.0 = (self.0 & !(1 << 8)) | (val << 8);
    }

    pub fn set_paddr(&mut self, addr : PhysAddr) {
        self.0 = addr.0 | self.0 & 0xfff;
    }

    pub fn get_paddr(&self) -> PhysAddr {
        PhysAddr(self.0 & !0xfff)
    }
}

/// A Page Directory
#[derive(Debug)]
pub struct PageDirectory {
    table : PhysAddr,
}

impl PageDirectory {
    /// Allocate a new `PageDirectory`
    pub fn new() -> Self {
        let page = unsafe { PhysMem::alloc_phys_zeroed() };
        Self {
            table : page,
        }
    }
    
    /// Create a `PageDirectory` from a physical address
    pub fn from_paddr(paddr : PhysAddr) -> Self {
        Self {
            table : paddr,
        }
    }

    /// Update the entry at `index` to make it `entry`
    fn set_entry(&self, index : usize, entry : u32) {
        let entry_paddr = PhysAddr(self.table.0 + 
                                   (index * size_of::<u32>()) as u32);
        let entry_vaddr = PhysMem::translate(entry_paddr, size_of::<u32>());
        unsafe {
            core::ptr::write(entry_vaddr as *mut u32, entry);
        }
    }

    /// Get the PDE at `index`
    fn get_entry(&self, index : usize) -> PageDirectoryEntry {
        let entry_paddr = PhysAddr(self.table.0 + 
                                   (index * size_of::<u32>()) as u32);
        let entry_vaddr = PhysMem::translate(entry_paddr, size_of::<u32>());
        let entry = unsafe {
            PageDirectoryEntry::new(core::ptr::read(entry_vaddr as *const u32))
        };
        entry
    }

    /// Create a page table entry at `vaddr` of length `size` bytes
    pub fn map(&self, vaddr : VirtAddr, size : usize, write : bool, 
                      user : bool) {
        
        let end_vaddr = vaddr.0 + (size as u32);

        // Iterate over all pages in the mapping 
        for vaddr in (vaddr.0..end_vaddr).step_by(PAGE_SIZE) {
            // Alloc a new physical page
            let page = unsafe { PhysMem::alloc_phys() };
            // Create a ptb entry corresponding to the allocated page
            let new_ptb_entry = PageTableEntry::new(
                page.0 | PAGE_PRESENT |
                if write { PAGE_WRITE } else { 0 } |
                if user { PAGE_USER } else { 0 }
            );
            // Add this mapping to the page table 
            unsafe {
                self.map_raw(VirtAddr(vaddr), new_ptb_entry.0);
            }
        }
    }

    /// Map a `vaddr` to a raw page table entry `raw`
    pub unsafe fn map_raw(&self, vaddr : VirtAddr, raw : u32) {
        let pgd_index = ((vaddr.0 >> 22) & 0x3ff) as usize;
        let ptb_index = ((vaddr.0 >> 12) & 0x3ff) as usize;

        let mut entry = self.get_entry(pgd_index);

        // If the entry is not present, allocate a blank page table and update
        // the corresponding PDE
        if entry.0 & PAGE_PRESENT == 0 {
            let new_ptb = PhysMem::alloc_phys_zeroed();
            //println!("allocating new pt at {:#x}", new_ptb.0);

            let new_pgd_entry = PageDirectoryEntry::new(
                new_ptb.0 | PAGE_PRESENT | PAGE_WRITE | PAGE_USER); 

            self.set_entry(pgd_index, new_pgd_entry.0);
            entry = new_pgd_entry;
        }
        
        // Get the page table from the entry paddr
        let ptb = PageTable::from_paddr(entry.get_paddr());
        
        // Update the entry
        ptb.set_entry(ptb_index, raw);
    }

    /// Return the physical address of this page table directory
    pub fn get_paddr(&self) -> PhysAddr {
        self.table
    }

    /// Translate a `vaddr` into its mapping components in the `self` page
    /// directory
    pub fn translate(&self, vaddr : VirtAddr) -> Mapping {
        let mut ret = Mapping {
            pde :  None,
            pte :  None,
            page : None,
        };

        // Compute pde / pte indicies
        let pde_index = ((vaddr.0 >> 22) & 0x3ff) as usize;
        let pte_index = ((vaddr.0 >> 12) & 0x3ff) as usize;
        
        ret.pde = Some(PhysAddr(self.table.0 + 
                                (pde_index * size_of::<u32>()) as u32));

        // Get the pde
        let pde = self.get_entry(pde_index);
        ret.pte = Some(pde.get_paddr());

        let ptb = PageTable::from_paddr(pde.get_paddr());

        // Get the pte
        let pte = ptb.get_entry(pte_index);
        ret.page = Some(pte.get_paddr());
        
        ret
    }
}

/// A Page Table
pub struct PageTable {
    table : PhysAddr,
}

impl PageTable {
    /// Create a `PageTable` from `paddr`
    fn from_paddr(paddr : PhysAddr) -> Self {
        Self {
            table : paddr, 
        }
    }

    /// Update the entry at `index` to make it `entry`
    fn set_entry(&self, index : usize, entry : u32) {
        let entry_paddr = PhysAddr(self.table.0 + 
                                   (index * size_of::<u32>()) as u32);
        let entry_vaddr = PhysMem::translate(entry_paddr, size_of::<u32>());
        unsafe {
            core::ptr::write(entry_vaddr as *mut u32, entry);
        }
    }
    
    /// Get the PTE at `index`
    fn get_entry(&self, index : usize) -> PageTableEntry {
        let entry_paddr = PhysAddr(self.table.0 + 
                                   (index * size_of::<u32>()) as u32);
        let entry_vaddr = PhysMem::translate(entry_paddr, size_of::<u32>());
        let entry = unsafe {
            PageTableEntry::new(core::ptr::read(entry_vaddr as *const u32))
        };
        entry
    }
}
