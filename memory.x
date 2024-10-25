/* Specify the memory areas */
MEMORY
{
  FLASH (rx)                 : ORIGIN = 0x08000000, LENGTH = 0x100000
  RAM (xrw)                 : ORIGIN = 0x20000000, LENGTH = 0x30000
}