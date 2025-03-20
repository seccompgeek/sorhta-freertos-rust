MEMORY
{
  /* S32G3 has 4 MB SRAM memory
     ARM Trusted Firmware would load our image at 0x80000000 */
  RAM : ORIGIN = 0xE0000000, LENGTH = 4M
}