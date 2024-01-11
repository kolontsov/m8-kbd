MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100 - 0x1000  /* first page is bootloader, last sector (16 pages) is NVRAM */
    NVRAM : ORIGIN = 0x10000000 + 2048K - 0x1000, LENGTH = 0x1000 /* reserve last sector (16 pages) for NVRAM */
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2

    .nvram ORIGIN(NVRAM) :
    {
        KEEP(*(.nvram));
    } > NVRAM
} INSERT BEFORE .text;