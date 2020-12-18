MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 31K
  DATA : ORIGIN = 0x080070c00, LENGTH = 1K
  RAM : ORIGIN = 0x20000000, LENGTH = 4K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{
  .user_data :
    {
      . = ALIGN(4);
      KEEP(*(.user_data))
      . = ALIGN(4);
    } > DATA
}
