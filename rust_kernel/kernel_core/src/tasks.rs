//! Ring3 tasks, context switches, scheduling, multitasking

use crate::cpu::*;
use crate::segmem::*;
use crate::paging::*;
use crate::paging::virtmem::*;
use crate::paging::pagemem::*;
use crate::interrupts::InterruptContext;
use crate::interrupts::resume_from_intr;
use core::mem::size_of;
use core::arch::asm;
use crate::{print, println, PERIPHERALS};

/// Size in pages of the kernel stack for a task
const KERNEL_STACK_SIZE : usize = 1;

/// Size in pages of the user stack for a task
const USER_STACK_SIZE : usize = 1;

/// Size in pages of the user code for a task
const USER_CODE_SIZE : usize = 1;

/// Max number of tasks that can run simultaneously on the system
const MAX_TASKS : usize = 10;

/// Used to init the `TASKS` array
const INIT_TASK : Option<Task> = None;

/// Contains all running tasks
static mut TASKS : [Option<Task>; MAX_TASKS] = [INIT_TASK; MAX_TASKS];

/// Index of currently executed task
static mut CURRENT_TASK_IDX : usize = usize::MAX;

/// All information needed to represent a task
#[derive(Debug)]
pub struct Task {
    /// The name of the task
    name : [u8; 16],
    
    /// CR3 value
    vspace : VirtMem,

    /// Kernel stack top
    pub kernel_sp : u32,

    /// User stack top
    user_sp : u32,
}

impl Task {
    /// Create a new task
    pub fn new(name : &[u8], code_addr : fn()) {
        let orig_vspace = VirtMem::get_current();

        if name.len() > 16 {
            panic!("Task name len > 16");
        }
        let mut task_name : [u8 ; 16] = [0; 16];
        task_name[..name.len()].copy_from_slice(name);

        let mut vspace = VirtMem::new();

        setup_identity_mapping(&vspace);
        switch_vspace(&vspace);

        let kernel_stack = vspace.alloc_virt_pages(KERNEL_STACK_SIZE, 
                                                   true, false);
        println!("kernel_stack : {:#x}", kernel_stack.0);
        let mut kernel_sp = kernel_stack.0 + 
            (KERNEL_STACK_SIZE * PAGE_SIZE) as u32;

        let user_stack = vspace.alloc_virt_pages(USER_STACK_SIZE, true, true);
        println!("user_stack : {:#x}", user_stack.0);
        let user_sp = user_stack.0 + (USER_STACK_SIZE * PAGE_SIZE) as u32;
        println!("user sp : {:#x}", user_sp);

        let code_addr = code_addr as *const u32 as u32;

        // Map user code as user accessible in virtual memory
        vspace.map_raw(VirtAddr(code_addr),
            code_addr | PAGE_USER | PAGE_PRESENT);

        // Create a fake interrupt context. This intr context will be used
        // to call switch_to() on this task and jump to userland
        let mut context = InterruptContext::default();
        context.frame.ip = code_addr;
        context.frame.cs = 0x18 | 3;
        context.frame.eflags = 0x200; // To enable interrupts on context switch
        context.frame.sp = user_sp;
        context.frame.ss = 0x20 | 3;

        // Push the "fake" interrupt context
        kernel_sp -= size_of::<InterruptContext>() as u32;
        unsafe { core::ptr::copy(&context, kernel_sp as *mut _, 
                                 size_of::<InterruptContext>()); }

        // Push the address of resume_from_intr
        kernel_sp -= size_of::<u32>() as u32;
        unsafe { core::ptr::write(kernel_sp as *mut _, 
                                  resume_from_intr as *const u32 as u32); }
        
        // Push padding values
        for _ in 0..3 {
            // Push the address of resume_from_intr
            kernel_sp -= size_of::<u32>() as u32;
            unsafe { core::ptr::write(kernel_sp as *mut _, 
                                      0 as *const u32 as u32); }
        }

        // Push user data segment selector
        kernel_sp -= size_of::<u32>() as u32;
        unsafe { core::ptr::write(kernel_sp as *mut _, 
                                  0x20 | 3 as u32); }
        
        // Find an empty task spot 
        let empty_spot = unsafe {
            TASKS.iter().position(|x| x.is_none())
                .expect("Too many running tasks")
        };
        
        let task = Self {
            name : task_name,
            vspace : vspace,
            kernel_sp : kernel_sp,
            user_sp : user_sp,
        };

        // Add the task to the TASKS array
        unsafe { TASKS[empty_spot] = Some(task); }
        switch_vspace(&orig_vspace);
    }
}

/// Switch task context from `prev` to `next`
pub fn switch_to(prev : &Task, next : &Task) {
    unsafe { 
        // Update the esp0 field of the TSS
        TSS.update_esp0(next.kernel_sp + 20 + 
                        size_of::<InterruptContext>() as u32);
    
        asm!("mov eax, ds    // Save data segment registers on kernel stack
              push eax
              
              cmp ecx, edx
              je 2f

              mov dword ptr [edi], esp  // Save task kernel_sp before switching

              2:
              mov cr3, edx   // Switch vspace

              mov esp, {}    // Switch kernel stack

              pop eax        // Restore data segment registers
              mov ds, ax
              mov es, ax
              mov gs, ax
              mov fs, ax
             ", 
             in(reg) next.kernel_sp,
             in("edi") &prev.kernel_sp,
             in("ecx") prev.vspace.get_pgd_paddr().0,
             in("edx") next.vspace.get_pgd_paddr().0,
        );
    }
}

/// Find the next task to execute in the `TASKS` array
#[inline(never)]
pub fn schedule() {
    unsafe {
        let prev_task;
        if CURRENT_TASK_IDX == usize::MAX {
            prev_task = TASKS[0].as_ref().unwrap();
        } else {
            prev_task = TASKS[CURRENT_TASK_IDX].as_ref().unwrap();
        }

        // Find the next task in the task array
        loop {
            CURRENT_TASK_IDX = (CURRENT_TASK_IDX + 1) % MAX_TASKS;
            if !TASKS[CURRENT_TASK_IDX].is_none() {
                break;
            }
        }

        switch_to(prev_task, TASKS[CURRENT_TASK_IDX].as_ref().unwrap());
    }
}

/*
/// Switch to Ring3 and execute the code at `code_addr`
#[inline(never)]
pub fn enter_ring3_task(code_addr : fn()) {

    let kernel_stack = alloc_virt_pages(KERNEL_STACK_SIZE / PAGE_SIZE);
    set_kernel_stack(kernel_stack.0);

    let user_stack = alloc_virt_pages(USER_STACK_SIZE / PAGE_SIZE);

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
        in("ecx") user_stack,
        options(att_syntax));
    }
}
*/

#[inline]
pub fn exit_task() {
    unsafe {
        set_esp(TSS.esp0);
        asm!("ret");
    }
}
