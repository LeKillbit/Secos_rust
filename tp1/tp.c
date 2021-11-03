/* GPLv2 (c) Airbus */
#include <debug.h>
#include <info.h>
#include <segmem.h>

extern info_t *info;

void tp()
{
    gdt_reg_t gdt;
    get_gdtr(gdt);
    debug("limit : %x\n", gdt.limit);
    debug("base : %x\n", gdt.addr);
    for (int i = 0; i < gdt.limit; i++)
        debug("desc : 0x%llx\n", gdt.desc[i]);

    set_cs(0x8);

}
