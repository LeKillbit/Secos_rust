OUTPUT_FORMAT("elf32-i386","elf32-i386","elf32-i386");
OUTPUT_ARCH("i386")

ENTRY(entry)

PHDRS
{
   phboot  PT_LOAD FLAGS (7);
   phstack PT_LOAD FLAGS (6);
   phsetup PT_LOAD FLAGS (7);
}

SECTIONS
{
   . = 0x300000;
   .mbh          : { KEEP(*(.mbh)) . = ALIGN(4); } : phboot
   .kernel_stack : { KEEP(*(.kernel_stack))      } : phstack

   __kernel_start__ = .;

   .idt_jmp  : { KEEP(*(.idt_jmp))               } : phsetup
   .text     : { *(.text)                        } : phsetup
   .rodata   : { *(.rodata)                      } : phsetup
   .data     : { *(.data)                        } : phsetup
   .bss      : { *(.bss COMMON)                  } : phsetup
   /DISCARD/ : { *(.note* .indent .comment)      } : phsetup
   .user_task ALIGN(0x1000) : 
   { 
        KEEP(*(.user_task)) . = ALIGN(0x1000); 
   } : phsetup

   __kernel_end__ = .;

   . = 0x400000;
   __base_allocator__ = .;
}
