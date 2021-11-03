use crate::cpu::*;
use crate::segmem::*;

/// Symbol defined in linker.lds
/// This where all usermode stacks belongs
extern {
    static __user_stack_start__ : u32;
}

/// Switch to Ring3 and execute the code at `code_addr`
#[inline(never)]
pub fn enter_ring3_task(code_addr : fn()) {

    set_kernel_stack(get_esp());

    let task_stack = alloc_stack();

    unsafe {
        asm!("
            cli                         // Disable interrupts
            mov ${data_selector}, %ax   // Set the data selectors
            mov %ax, %ds                // to user data segment
            mov %ax, %es
            mov %ax, %fs
            mov %ax, %gs
                                        // Create the frame on the stack
            push ${data_selector}       // ss
            push %ecx                   // esp
            pushf                       // eflags
            pop %eax                
            or $0x200, %eax             // (re-enable interrupts
            push %eax                   //  when iret is executed)
            push ${code_selector}       // cs
            push %edx                   // eip
            iret                        // jump to ring3
        ",
        data_selector = const 0x20 | 3,
        code_selector = const 0x18 | 3,
        in("edx") code_addr,
        in("ecx") task_stack,
        options(att_syntax));
    }
}

#[inline]
pub fn exit_task() {
    unsafe {
        set_esp(TSS.esp0);
        asm!("ret");
    }
}

/// Alloc a `0x1000` bytes stack for a user task
fn alloc_stack() -> u32 {
    let start : u32 = unsafe { &__user_stack_start__ as *const u32 as u32};
    start
}
