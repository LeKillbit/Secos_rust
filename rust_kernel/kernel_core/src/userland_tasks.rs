#[no_mangle]
#[link_section=".user_task"]
pub fn task1() {
    print("hello from userland task1!\n");
    loop {
        print("task 1\n");
    }
}

#[no_mangle]
#[link_section=".user_task"]
pub fn task2() {
    print("hello from userland task2!\n");
    loop {
        print("task 2\n");
    }
}

#[no_mangle]
#[link_section=".user_task"]
#[inline(never)]
fn print(data : &str) {
    unsafe {
        asm!("mov eax, 2
              int 0x80",
              in("ecx") data.as_ptr(),
              in("edx") data.len());
    }
}

