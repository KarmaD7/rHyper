OUTPUT_ARCH(aarch64)

BASE_ADDRESS = 0x0000000040080000;

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text : {
        stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .dtb : {
        sdtb = .;
        *(.dtb .dtb.*)
        . = ALIGN(4K);
        edtb = .;
    }

    .initramfs : {
        sinitramfs = .;
        *(.initramfs .initramfs.*)
        . = ALIGN(4K);
        einitramfs = .;
    }

    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : {
        sdata = .;
        *(.data.boot_page_table)
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
        . = ALIGN(4K);
        edata = .;
    }

    .bss : {
        boot_stack = .;
        *(.bss.stack)
        . = ALIGN(4K);
        boot_stack_top = .;

        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }

    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame) *(.eh_frame_hdr)
    }
}
