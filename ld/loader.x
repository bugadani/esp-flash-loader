/* Shared loader, requires MEMORY definitions each chip */

SECTIONS {
    . = 0x0;

    /* Section for code and readonly data, specified by flashloader standard. */
    PrgCode : {
        . = ALIGN(4);

        /* The KEEP is necessary to ensure that the
         * sections don't get garbage collected by the linker.
         * 
         * Because this is not a normal binary with an entry point,
         * the linker would just discard all the code without the
         * KEEP statement here.
         */

        KEEP(*(.text))
        KEEP(*(.text.*))
        
        . = ALIGN(4);
    } > IRAM

    PrgData : {
        . = ALIGN(4);

        *(COMMON)

        KEEP(*(.rodata))
        KEEP(*(.rodata.*))

        *(.data .data.*)
        *(.sdata .sdata.*)

        *(.bss .bss.*)
        *(.sbss .sbss.*)

        . = ALIGN(4);
    } > DRAM

    /* Description of the flash algorithm */
    DeviceData : 
    {
        /* The device data content is only for external tools,
         * and usually not referenced by the code.
         *
         * The KEEP statement ensures it's not removed by accident.
         */
        KEEP(*(DeviceData))
    } > INFO
}


