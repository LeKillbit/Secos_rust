[bits 32]

section .kernel_stack align=16 nobits alloc write
resb 0x2000

; section .user_stack align=16 nobits alloc write
; resb 0x6000

section .text

extern __kernel_start__
extern rust_main

global entry
entry:
    cli
    mov     esp, __kernel_start__
    push    0
    popf
    mov     ecx, ebx
    call    rust_main

halt:
    hlt
    jmp     halt
