use crate::cpu::*;
use crate::{println, print, PERIPHERALS};

/// Access rights for a GDT entry
pub const AccessPresent : u8 = 1 << 7;
pub const AccessRing0 : u8 = 0 << 5;
pub const AccessRing3 : u8 = 3 << 5;
pub const AccessSystem : u8 = 1 << 4;
pub const AccessExecutable : u8 = 1 << 3;
pub const AccessConforming : u8 = 1 << 2;
pub const AccessRW : u8 = 1 << 1;
pub const AccessAccessed : u8 = 1 << 0;

/// Flags for a GDT entry
/// Default : ByteGranularity
pub const FlagsPageGranularity : u8 = 1 << 3;
/// Default : Size16
pub const FlagsSize32 : u8 = 1 << 2;

const MAX_GDT_SIZE : usize = 8192;

static mut GDT_ENTRIES : [SegmentDescriptor; 6] = [ 
    SegmentDescriptor::null_descriptor(); 6
];

pub static mut TSS : TssEntry = TssEntry::default();

/// An entry in the TSS
#[repr(C)]
pub struct TssEntry {
    pub prev_tss : u32, // Used in hardware-based context switch
    pub esp0 : u32,     // Stack pointer to load when switching to kernel mode
    pub ss0 : u32,      // Stack segment to load when switching to kernel mode
    // Unused from here
    esp1 : u32,
    ss1 : u32,
    esp2 : u32,
    ss2 : u32,
    cr3 : u32,
    eip : u32,
	eflags : u32,
	eax : u32,
	ecx : u32,
	edx : u32,
	ebx : u32,
	esp : u32,
	ebp : u32,
	esi : u32,
	edi : u32,
	es : u32,
	cs : u32,
	ss : u32,
	ds : u32,
	fs : u32,
	gs : u32,
	ldt : u32,
	trap : u16,
	iomap_base : u16,
}

impl TssEntry {
    const fn default() -> Self {
        Self {
            prev_tss : 0, 
            esp0 : 0,     
            ss0 : 0,     
            esp1 : 0,
            ss1 : 0,
            esp2 : 0,
            ss2 : 0,
            cr3 : 0,
            eip : 0,
            eflags : 0,
            eax : 0,
            ecx : 0,
            edx : 0,
            ebx : 0,
            esp : 0,
            ebp : 0,
            esi : 0,
            edi : 0,
            es : 0,
            cs : 0,
            ss : 0,
            ds : 0,
            fs : 0,
            gs : 0,
            ldt : 0,
            trap : 0,
            iomap_base : 0,
        }
    }

    /// Update the esp0 field 
    pub fn update_esp0(&mut self, esp : u32) {
        self.esp0 = esp;
    }
}

/// Init the GDT
pub fn gdt_init() {
    let mut gdt_pointer = unsafe {
        GdtPointer {
            limit : (GDT_ENTRIES.len() as u16) * 8 - 1,
            base : GDT_ENTRIES.as_ptr() as u32,
        }
    };

    gdt_pointer.add_descriptor(1, SegmentDescriptor::kernel_code_desc());
    gdt_pointer.add_descriptor(2, SegmentDescriptor::kernel_data_desc());
    gdt_pointer.add_descriptor(3, SegmentDescriptor::user_code_desc());
    gdt_pointer.add_descriptor(4, SegmentDescriptor::user_data_desc());
    gdt_pointer.add_descriptor(5, SegmentDescriptor::tss_desc());

    set_gdt(&gdt_pointer);

    set_cs::<0x8>();
    set_ds(0x10);
    set_es(0x10);
    set_fs(0x10);
    set_gs(0x10);
    set_ss(0x10);

    // Set the tss stack segment to kernel data segment
    // This is unsafe because we are mutating a static variable
    unsafe { 
        TSS.ss0 = 0x10;
        TSS.esp0 = 0;

        /*TSS.cs = 0x8  | 3;
        TSS.ss = 0x10 | 3;
        TSS.ds = 0x10 | 3;
        TSS.es = 0x10 | 3;
        TSS.fs = 0x10 | 3;
        TSS.gs = 0x10 | 3;*/
    }

    flush_tss();
}

/// Switch the esp0 value in `TSS` 
#[inline]
pub fn set_kernel_stack(esp : u32) {
    unsafe {
        TSS.esp0 = esp;
    }
}

#[allow(unaligned_references)]
pub fn print_current_gdt() {
    let mut gdtp : GdtPointer = Default::default();
    get_gdt(&mut gdtp);
    println!("gdt size : {:#x}", gdtp.limit);
    println!("gdt base : {:#x}", gdtp.base);
    for i in 0..(gdtp.limit+1)/8 {
        let entry = unsafe { 
            &*(gdtp.base as *mut SegmentDescriptor).offset(i as isize)
        };
        println!("entry base : {:#x}", entry.get_base());
        println!("entry limit : {:#x}", entry.get_limit());
    }
}

/// Structure describing the GDT
#[repr(C, packed)]
pub struct GdtPointer {
    /// Size of the GDT - 1
    pub limit : u16,
    /// Linear address of the table itself
    pub base : u32,
}

impl Default for GdtPointer {
    fn default() -> Self {
        GdtPointer {
            limit : 0,
            base : 0,
        }
    }
}

impl GdtPointer {
    /// Add a descriptor to the gdt
    pub fn add_descriptor(&mut self, index : isize, 
                          descriptor : SegmentDescriptor) {
        if index < 0 || index > 8192 {
            panic!("invalid gdt index");
        } 
        unsafe {
            *(self.base as *mut SegmentDescriptor).offset(index) = descriptor;
        }
    }
}

/// An entry in the GDT, describes a Segment
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct SegmentDescriptor {
    pub limit1 : u16,
    pub base1 : u16,
    pub base2 : u8,
    pub access : u8,
    pub limit2_flags : u8,
    pub base3 : u8,
}

impl SegmentDescriptor {
    pub fn new(base : u32, limit : u32, access : u8, flags : u8) 
        -> Self {
        let mut sd = Self {
            limit1 : 0,
            base1 : 0,
            base2 : 0,
            access : access,
            limit2_flags : 0,
            base3 : 0,
        };

        sd.set_base(base);
        sd.set_limit(limit);
        sd.set_flags(flags);

        sd
    }

    pub const fn null_descriptor() -> Self {
        Self {
            limit1 : 0,
            base1 : 0,
            base2 : 0,
            access : 0,
            limit2_flags : 0,
            base3 : 0,
        }
    }

    fn kernel_code_desc() -> Self {
        Self::new(
            0x0,
            0xfffff,
            AccessPresent | AccessRing0 | AccessSystem | AccessExecutable |
            AccessRW,
            FlagsSize32 | FlagsPageGranularity
        )
    }

    fn kernel_data_desc() -> Self {
        Self::new(
            0x0,
            0xfffff,
            AccessPresent | AccessRing0 | AccessSystem | AccessRW,
            FlagsSize32 | FlagsPageGranularity
        )
    }

    fn user_code_desc() -> Self {
        Self::new(
            0x0,
            0xfffff,
            AccessPresent | AccessRing3 | AccessSystem | AccessExecutable |
            AccessRW,
            FlagsSize32 | FlagsPageGranularity
        )
    }

    fn user_data_desc() -> Self {
        Self::new(
            0x0,
            0xfffff,
            AccessPresent | AccessRing3 | AccessSystem | AccessRW,
            FlagsSize32 | FlagsPageGranularity
        )
    }

    fn tss_desc() -> Self {
        unsafe {
            Self::new(
                &TSS as *const _ as u32,
                core::mem::size_of::<TssEntry>() as u32,
                AccessAccessed | AccessExecutable | AccessPresent,
                0
            )
        }
    }

    fn set_flags(&mut self, flags : u8) {
        self.limit2_flags = self.limit2_flags & 0x0f |
            (flags & 0xf) << 4;
    }

    fn set_limit(&mut self, limit : u32) {
        if limit >> 20 != 0 {
            panic!("limit value too large");
        }
        self.limit1 = limit as u16;
        self.limit2_flags = self.limit2_flags & 0xf0 |
            ((limit >> 16) as u8) & 0x0f;
    }

    pub fn get_limit(&self) -> u32 {
        let mut limit : u32 = 0;
        limit |= self.limit1 as u32;
        limit |= ((self.limit2_flags as u32) & 0x0f) << 16;
        limit
    }

    fn set_base(&mut self, base : u32) {
        self.base1 = base as u16;
        self.base2 = (base >> 16) as u8;
        self.base3 = (base >> 24) as u8;
    }
    
    pub fn get_base(&self) -> u32 {
        let mut base : u32 = 0;
        base |= self.base1 as u32;
        base |= (self.base2 as u32) << 16;
        base |= (self.base3 as u32) << 24;
        base
    }
}
