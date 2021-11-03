use crate::cpu;

const PIC1_COMMAND : u16 = 0x20;
const PIC1_DATA : u16 = 0x21;

const PIC2_COMMAND : u16 = 0xa0;
const PIC2_DATA : u16 = 0xa1;

const ICW1_ICW4 : u8 = 0x01;           /* ICW4 (not) needed */
const ICW1_SINGLE : u8 = 0x02;         /* Single (cascade) mode */
const ICW1_INTERVAL4 : u8 = 0x04;      /* Call address interval 4 (8) */
const ICW1_LEVEL : u8 = 0x08;          /* Level triggered (edge) mode */
const ICW1_INIT : u8 = 0x10;           /* Initialization - required! */
 
const ICW4_8086 : u8 = 0x01;           /* 8086/88 (MCS-80/85) mode */
const ICW4_AUTO : u8 = 0x02;           /* Auto (normal) EOI */
const ICW4_BUF_SLAVE : u8 = 0x08;      /* Buffered mode/slave */
const ICW4_BUF_MASTER : u8 = 0x0C;     /* Buffered mode/master */
const ICW4_SFNM : u8 = 0x10;           /* Special fully nested (not) */

/// Remap the Programmable Interrupt Controllers to specified 
/// vector offsets : `offset1` for master PIC and `offset2` for slave PIC
pub fn pic_remap(offset1 : u8, offset2 : u8) {
    unsafe {
        // First init word (ICW1) : init the two PICS
        //      - ICW4 needed
        //      - cascade mode
        cpu::out8(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        cpu::out8(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);

        // Second init word (ICW2) : Vector offset for the PICS
        //      - remap IRQ[00-07] to IDT[offset1-offset1+7]
        //      - remap IRQ[08-15] to IDT[offset2-offset2+7] 
        cpu::out8(PIC1_DATA, offset1);
        cpu::out8(PIC2_DATA, offset2);

        // Third init word (ICW3) : Master / Slave wiring
        //      - tell master PIC that there is a slave at IRQ2
        //      - tell slave PIC its cascade identity
        cpu::out8(PIC1_DATA, 4);
        cpu::out8(PIC2_DATA, 2);

        // Fourth init word (ICW4) : Environment Info
        //      - x86 mode
        //      - normal EOI
        //      - not buffered
        //      - not fully nested
        cpu::out8(PIC1_DATA, ICW4_8086);
        cpu::out8(PIC2_DATA, ICW4_8086);
    }
}

