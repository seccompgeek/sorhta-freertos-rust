ENTRY(_start)

/*
 * S32G3 Memory Map with ARM Trusted Firmware:
 * - 0x34000000 - 0x3FFFFFFF: SRAM (64 MB)
 * - 0x80000000 - 0xFFFFFFFF: DRAM (2 GB)
 *
 * ATF loads us at 0xE0000000 in DRAM
 */

MEMORY
{
    RAM : ORIGIN = 0xE0000000, LENGTH = 16M
}

SECTIONS
{
    /* ATF will load us at 0xE0000000 */
    . = 0xE0000000;
    
    .text : {
        /* Make sure _start is at the beginning */
        *(.text.boot)       /* Boot code section */
        *(.text.exceptions) /* Exception vectors */
        *(.text .text.*)    /* Code sections */
        . = ALIGN(8);
    } > RAM
    
    .rodata : {
        *(.rodata .rodata.*) /* Read-only data */
        . = ALIGN(8);
    } > RAM
    
    .data : {
        *(.data .data.*)     /* Initialized data */
        . = ALIGN(8);
    } > RAM
    
    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)       /* Uninitialized data */
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > RAM
    
    /* Stack allocation - 64 KiB per core */
    . = ALIGN(4096);
    __stack_start = .;
    . += 0x10000;         /* 64 KiB stack space */
    __stack_end = .;      /* Define stack end symbol for ASM code */
    
    /* Heap allocation - 1 MiB */
    . = ALIGN(4096);
    _heap_start = .;
    . += 0x100000;        /* 1 MiB heap space */
    _heap_end = .;
    
    /DISCARD/ : {
        *(.comment)
        *(.gnu*)
        *(.note*)
        *(.eh_frame*)
    }
}