pub const MBH_MAGIC : u32 = 464367618;
pub const MBH_FLAGS : u32 = 3;

#[repr(C)]
pub struct MultibootInfo {
    pub flags : u32,

    mem_lower : u32,
    mem_upper : u32,

    boot_device : u32,

    cmdline : u32,

    mods_count : u32,
    mods_addr : u32,

    syms : Symbols,

    pub mmap_length : u32,
    pub mmap_addr : u32,

    drives_length : u32,
    drives_addr : u32,

    config_table : u32,

    boot_loader_name : u32,
    
    apm_table : u32,

    vbe_control_info : u32,
    vbe_mode_info : u32,
    vbe_mode : u32,
    vbe_interface_seg : u32,
    vbe_interface_off : u32,
    vbe_interface_len : u32,

    framebuffer_addr : u32,
    framebuffer_pitch : u32,
    framebuffer_width : u32,
    framebuffer_height : u32,
    framebuffer_bpp : u8,
    framebuffer_type : u8,

    framebuffer_table : FramebufferTable,
}

#[repr(C)]
union Symbols {
    aout: AOutSymbols,
    elf: ElfSymbols,
    //_bindgen_union_align: [u32; 4usize],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct AOutSymbols {
    tabsize: u32,
    strsize: u32,
    addr: u32,
    reserved: u32,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ElfSymbols {
    num: u32,
    size: u32,
    addr: u32,
    shndx: u32,
}

#[repr(C)]
struct FramebufferTable {
    addr : u64,
    pitch : u64,
    width : u64,
    height : u64,
    bpp : u8,
    ty : u8,
    color_info : ColorInfo,
}

#[repr(C)]
union ColorInfo {
    palette: ColorInfoPalette,
    rgb: ColorInfoRgb,
    _union_align: [u32; 2usize],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ColorInfoPalette {
    palette_addr: u32,
    palette_num_colors: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ColorInfoRgb {
    red_field_position: u8,
    red_mask_size: u8,
    green_field_position: u8,
    green_mask_size: u8,
    blue_field_position: u8,
    blue_mask_size: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct multiboot_mmap_entry {
    pub size : u32,
    pub addr : u64,
    pub len : u64,
    pub ty : u32,
}
