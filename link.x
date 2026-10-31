PATH_SIZE   = 1024;
PARAMS_SIZE =  256;
STACK_SIZE  = 1024*64;

SECTIONS {
  hdr : {
    LONG(0x554e454D);
    LONG(0x31305445);
    LONG(1);                                               // Header version
    LONG(kol_main);                                        // Program start
    LONG(END);                                             // Image size
    LONG(FILE_END + PATH_SIZE + PARAMS_SIZE + STACK_SIZE); // Required amount of memory
    LONG(FILE_END + PATH_SIZE + PARAMS_SIZE + STACK_SIZE); // Stack
    LONG(FILE_END + PATH_SIZE);                            // Boot params
    LONG(FILE_END);                                        // Application path
  }

  .text : {
    *(.text)
    *(.text.*)
  }

  END = .;

  .data ALIGN(16) : {
    *(.rodata.*)
    *(const)
    *(CONST)
    *(.data)
    *(data)
  }

  .bss ALIGN(16) : {*(.bss .bss.* COMMON)}

  FILE_END = .;

  /DISCARD/ : { *(.eh_frame*) }
}
