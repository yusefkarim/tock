MEMORY
{
  rom (rx)  : ORIGIN = 0x00020000, LENGTH = 182K
  prog (rx) : ORIGIN = 0x00050000, LENGTH = 832K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld
