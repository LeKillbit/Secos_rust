use core::arch::asm;

#[no_mangle]
#[link_section=".user_task"]
pub fn task1() {
    mmap_shared(0x1000_0000, 0);
    print("hello from userland task1!\n");
    let mut ctr : u32 = 0;
    loop {
        ctr += 1;
        unsafe { 
            core::ptr::write_volatile(0x1000_0000 as *mut u32, ctr); 
        }
        //print("task 1 : ");
        //print_number(tmp);
    }
}

#[no_mangle]
#[link_section=".user_task"]
pub fn task2() {
    mmap_shared(0x2000_0000, 0);
    print("hello from userland task2!\n");
    let mut num : u32 = 0;
    loop {
        let tmp : u32 = unsafe {
            core::ptr::read_volatile(0x2000_0000 as *const u32)
        };
        if tmp != num {
            num = tmp;
            print("task 2 : ");
            print_number(num);
        }
    }
}

#[no_mangle]
#[link_section=".user_task"]
#[inline(never)]
fn print(data : &str) {
    write(data.as_ptr(), data.len());
}

#[no_mangle]
#[link_section=".user_task"]
#[inline(never)]
fn print_number(num : u32) {
    unsafe {
        asm!("mov eax, 3
              int 0x80",
              in("ecx") num);
    }
}

#[no_mangle]
#[link_section=".user_task"]
#[inline(never)]
fn write(addr : *const u8, len : usize) {
    unsafe {
        asm!("mov eax, 2
              int 0x80",
              in("ecx") addr,
              in("edx") len);
    }
}

/// Wrapper to use the mmap_shared syscall
#[no_mangle]
#[link_section=".user_task"]
#[inline(never)]
fn mmap_shared(addr : u32, id : usize) {
    unsafe {
        asm!("mov eax, 10
              int 0x80",
              in("ecx") addr,
              in("edx") id as u32);
    }
}

