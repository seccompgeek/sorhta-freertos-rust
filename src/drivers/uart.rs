use core::arch::{asm, global_asm};
use core::{fmt, ptr};
use core::ptr::{null_mut, read_volatile, write_volatile};
use crate::arch::s32g3::{
    CONSOLE_FLAG_BOOT, CONSOLE_FLAG_CRASH, CONSOLE_T_BASE, CONSOLE_T_FLAGS, CONSOLE_T_FLUSH, CONSOLE_T_GETC, CONSOLE_T_PUTC, LDIV_MULTIPLIER, LINCR1_INIT, LINCR1_MME, LINFLEX_BDRL, LINFLEX_LINCR1, LINFLEX_LINFBRR, LINFLEX_LINIBRR, LINFLEX_LINSR, LINFLEX_UARTCR, LINFLEX_UARTPTO, LINFLEX_UARTSR, LINSR_LINS_INITMODE, LINSR_LINS_MASK, UARTCR_OSR_SHIFT, UARTCR_OSR_WIDTH, UARTCR_PC0, UARTCR_PC1, UARTCR_RFBM, UARTCR_ROSE, UARTCR_RXEN, UARTCR_TFBM, UARTCR_TFC, UARTCR_TXEN, UARTCR_UART, UARTCR_WL0, UARTSR_DTF, UART_BASE, UART_BAUD_RATE, UART_CLOCK_HZ
};

pub const EINVAL: usize = usize::MAX;
pub const LINFLEX: &str = "linflex";

#[repr(C)]
struct Console {
    next: *mut Console,
    flags: u64,  // Using u64 for u_register_t to align properly on AArch64
    putc: unsafe extern "C" fn(character: i32, console: &Console) -> i32,
    flush: unsafe extern "C" fn(console: &Console),
    base: usize,
    // Additional private driver data may follow here
}

pub const DEFAULTCONSOLE: Console = Console {next: null_mut(), flags: 0, putc: console_linflex_putc, flush: console_linflex_flush, base: UART_BASE};


/// uint32_t get_ldiv_mult(uintptr_t baseaddr, uint32_t clock,
///                        uint32_t baud, console_t *console);
///
/// Clobber list : x0 - x6
/// Out x4: LDIV multiplier
#[no_mangle]
unsafe extern "C" fn get_ldiv_mult(
    _baseaddr: usize,
    _clock: u32, 
    _baud: u32, 
    _console: &Console
) -> u32 {
    asm!(
        "ldr     w4, [x0, {LINFLEX_UARTCR}]",
        "mov     w5, w4",

        // Prepare choices in w5 and w6
        "ubfx    x5, x5, {UARTCR_OSR_SHIFT}, {UARTCR_OSR_WIDTH}",
        "mov     w6, {LDIV_MULTIPLIER}",

        "and     w4, w4, {UARTCR_ROSE}",
        "cmp     w4, #0x0",
        "csel    w4, w5, w6, ne",
        "ret",
        
        LINFLEX_UARTCR = const LINFLEX_UARTCR,
        UARTCR_OSR_SHIFT = const UARTCR_OSR_SHIFT,
        UARTCR_OSR_WIDTH = const UARTCR_OSR_WIDTH,
        UARTCR_ROSE = const UARTCR_ROSE,
        LDIV_MULTIPLIER = const LDIV_MULTIPLIER,
        
        options(noreturn)
    );
}

/// void linflex_set_brg(uintptr_t baseaddr, uint32_t clock
///                      uint32_t baud, console_t *console);
///
/// Clobber list : x0 - x7, x13
#[no_mangle]
unsafe extern "C" fn linflex_set_brg(
    _baseaddr: usize,
    _clock: u32,
    _baud: u32,
    _console: &Console
) {
    asm!(
        "mov     x13, x30",
        "bl      get_ldiv_mult",
        "mov     x30, x13",

        // (x4) dividr = baudrate * ldiv_mult
        "mul     x4, x4, x2",
        // (x5) divisr = clock rate
        "mov     x5, x1",
        // (x6) ibr = divisr / dividr
        "udiv    x6, x5, x4",
        // (x7) fbr = divisr % dividr
        "msub    x7, x6, x4, x5",
        // fbr *= 16 / dividr
        "lsl     x7, x7, #4",
        "udiv    x7, x7, x4",
        // fbr &= 0xf
        "and     w7, w7, #0xf",
        "str     w6, [x0, {LINFLEX_LINIBRR}]",
        "str     w7, [x0, {LINFLEX_LINFBRR}]",
        "ret",
        
        LINFLEX_LINIBRR = const LINFLEX_LINIBRR,
        LINFLEX_LINFBRR = const LINFLEX_LINFBRR,
        
        options(noreturn)
    );
}

/// int console_linflex_core_init(uintptr_t baseaddr, uint32_t clock,
///                               uint32_t baud);
///
/// In:  x0 - Linflex base address
///      x1 - clock frequency
///      x2 - baudrate
/// Out: x0 - 1 on success, 0 on error
/// Clobber list : x0 - x7, x13 - x14
#[no_mangle]
unsafe extern "C" fn console_linflex_core_init(
    _baseaddr: usize,
    _clock: u32,
    _baud: u32
) -> i32 {
    asm!(
        // Set master mode and init mode
        "mov     w4, {LINCR1_INIT}",
        "str     w4, [x0, {LINFLEX_LINCR1}]",
        "mov     w4, {LINCR1_MME_INIT}",
        "str     w4, [x0, {LINFLEX_LINCR1}]",

        // Wait for init mode entry
        "2:",
        "ldr     w4, [x0, {LINFLEX_LINSR}]",
        "and     w4, w4, {LINSR_LINS_MASK}",
        "cmp     w4, {LINSR_LINS_INITMODE}",
        "b.ne    2b",

        // Set UART bit
        "mov     w4, {UARTCR_UART}",
        "str     w4, [x0, {LINFLEX_UARTCR}]",

        // Call linflex_set_brg with NULL console pointer
        "mov     x14, x30",
        "mov     x3, #0",          // Set console pointer to null
        "bl      linflex_set_brg", 
        "mov     x30, x14",

        // Set preset timeout register value
        "mov     w4, #0xf",
        "str     w4, [x0, {LINFLEX_UARTPTO}]",

        // 8-bit data, no parity, Tx/Rx enabled, UART mode
        "mov     w4, {UARTCR_CONFIG}",
        "str     w4, [x0, {LINFLEX_UARTCR}]",

        // End init mode
        "ldr     w4, [x0, {LINFLEX_LINCR1}]",
        "bic     w4, w4, {LINCR1_INIT}",
        "str     w4, [x0, {LINFLEX_LINCR1}]",
        
        // Return success
        "mov     w0, #1",
        "ret",
        
        // Constants
        LINCR1_INIT = const LINCR1_INIT,
        LINFLEX_LINCR1 = const LINFLEX_LINCR1,
        LINCR1_MME_INIT = const (LINCR1_MME | LINCR1_INIT),
        LINFLEX_LINSR = const LINFLEX_LINSR,
        LINSR_LINS_MASK = const LINSR_LINS_MASK,
        LINSR_LINS_INITMODE = const LINSR_LINS_INITMODE,
        UARTCR_UART = const UARTCR_UART,
        LINFLEX_UARTCR = const LINFLEX_UARTCR,
        LINFLEX_UARTPTO = const LINFLEX_UARTPTO,
        UARTCR_CONFIG = const (UARTCR_PC1 | UARTCR_RXEN | UARTCR_TXEN | UARTCR_PC0 | UARTCR_WL0 | UARTCR_UART | UARTCR_RFBM | UARTCR_TFBM),
        
        options(noreturn)
    );
}

/// int console_linflex_register(uintptr_t baseaddr, uint32_t clock,
///                              uint32_t baud, console_t *console);
///
/// Function to initialize and register the console.
/// The caller needs to pass an empty console_linflex_t
/// structure in which *MUST* be allocated in
/// persistent memory (e.g. a global or static local
/// variable, *NOT* on the stack).
/// In:  x0 - Linflex base address
///      x1 - clock frequency
///      x2 - baudrate
///      x3 - pointer to empty console_t structure
/// Out: x0 - 1 on success, 0 on error
/// Clobber list : x0 - x7, x13 - x15

#[no_mangle]
/// int console_linflex_register(uintptr_t baseaddr, uint32_t clock,
///                              uint32_t baud, console_t *console);
///
/// Function to initialize and register the console.
/// The caller needs to pass an empty console_linflex_t
/// structure in which *MUST* be allocated in
/// persistent memory (e.g. a global or static local
/// variable, *NOT* on the stack).
/// In:  x0 - Linflex base address
///      x1 - clock frequency
///      x2 - baudrate
///      x3 - pointer to empty console_t structure
/// Out: x0 - 1 on success, 0 on error
/// Clobber list : x0 - x7, x13 - x15
#[no_mangle]
unsafe extern "C" fn console_linflex_register(
    _baseaddr: usize,
    _clock: u32,
    _baud: u32,
    _console: &Console
) -> i32 {
    asm!(
        // Save return address
        "mov     x15, x30",
        "bl      console_linflex_core_init",
        "mov     x30, x15",

        // Populate the base address
        "str     x0, [x3, {CONSOLE_T_BASE}]",

        // Set up return value (pointer to console structure)
        "mov     x0, x3",
        
        // Inline expanded finish_console_register macro with linflex, putc=1, getc=0, flush=1
        
        // For putc=1
        "adrp    x1, console_linflex_putc",
        "add     x1, x1, :lo12:console_linflex_putc",
        "str     x1, [x0, {CONSOLE_T_PUTC}]",
        
        // For flush=1
        "adrp    x1, console_linflex_flush",
        "add     x1, x1, :lo12:console_linflex_flush",
        "str     x1, [x0, {CONSOLE_T_FLUSH}]",
        
        // Set console flags
        "mov     x1, {CONSOLE_FLAGS}",
        "str     x1, [x0, {CONSOLE_T_FLAGS}]",
        
        // Register the console
        //"b       console_register",
        
        // Constants
        CONSOLE_T_BASE = const CONSOLE_T_BASE,
        CONSOLE_T_PUTC = const CONSOLE_T_PUTC,
        CONSOLE_T_FLUSH = const CONSOLE_T_FLUSH,
        CONSOLE_T_FLAGS = const CONSOLE_T_FLAGS,
        CONSOLE_FLAGS = const (CONSOLE_FLAG_BOOT | CONSOLE_FLAG_CRASH),
        
        options(noreturn)
    );
}

/// int console_linflex_core_flush(uintptr_t baseaddr);
///
/// Loop while the TX fifo is not empty, depending on the selected UART mode.
///
/// In:  x0 - Linflex base address
/// Clobber list : x0 - x1
#[no_mangle]
unsafe extern "C" fn console_linflex_core_flush(
    _baseaddr: usize
) -> i32 {
    asm!(
        // Check if UART is in buffer mode
        "ldr     w1, [x0, {LINFLEX_UARTCR}]",
        "and     w1, w1, {UARTCR_TFBM}",

        "cmp     w1, #0x0",
        // If not in buffer mode, skip to exit
        "b.eq    3f",

        // Loop while TX FIFO is not empty
        "2:", //tfc_2loop
        "ldr     w1, [x0, {LINFLEX_UARTCR}]",
        "and     w1, w1, {UARTCR_TFC}",
        "cmp     w1, #0",
        "b.ne    2b",

        "3:", //exit_flush
        // Return success
        "mov     x0, #0",
        "ret",
        
        // Constants
        LINFLEX_UARTCR = const LINFLEX_UARTCR,
        UARTCR_TFBM = const UARTCR_TFBM,
        UARTCR_TFC = const UARTCR_TFC,
        
        options(noreturn)
    );
}

/// int console_linflex_core_putc(int c, uintptr_t baseaddr);
///
/// Out: w0 - printed character on success, < 0 on error.
/// Clobber list : x0 - x3
#[no_mangle]
unsafe extern "C" fn console_linflex_core_putc(
    _c: i32,
    _baseaddr: usize
) -> i32 {
    asm!(
        // Check if baseaddr is NULL
        "cbz    x1, 7f",

        // Check if character is newline
        "cmp    w0, '\\n'",
        "b.ne   2f",

        // Print '\r\n' for each '\n'
        "mov    x0, '\\r'",
        "mov    x14, x30",
        "bl     console_linflex_core_putc",
        "mov    x30, x14",
        "mov    x0, '\\n'",

        "2:", //print_char
        // Check if UART is in buffer mode
        "ldr    w2, [x1, {LINFLEX_UARTCR}]",
        "and    w2, w2, {UARTCR_TFBM}",
        "cmp    w2, #0x0",
        "b.eq   4f",

        "3:", //fifo_mode
        // UART is in FIFO mode
        "ldr    w2, [x1, {LINFLEX_UARTSR}]",
        "and    w2, w2, {UARTSR_DTF}",
        "cmp    w2, #0",
        "b.ne   3b",

        "strb   w0, [x1, {LINFLEX_BDRL}]",
        "b      6f",

        "4:", //buffer_mode
        "strb   w0, [x1, {LINFLEX_BDRL}]",

        "5:", //buffer_loop
        "ldr    w2, [x1, {LINFLEX_UARTSR}]",
        "and    w3, w2, {UARTSR_DTF}",
        "cmp    w3, #0",
        "b.eq   5b",

        // In Buffer Mode the DTFTFF bit of UARTSR register
        // has to be set in software
        "mov    w2, {UARTSR_DTF}",
        "str    w2, [x1, {LINFLEX_UARTSR}]",

        "6:", //no_error
        "mov    x0, #0",
        "ret",

        "7:", //putc_error
        "mov    x0, {EINVAL_NEG}",
        "ret",
        
        // Constants
        LINFLEX_UARTCR = const LINFLEX_UARTCR,
        UARTCR_TFBM = const UARTCR_TFBM,
        LINFLEX_UARTSR = const LINFLEX_UARTSR,
        UARTSR_DTF = const UARTSR_DTF,
        LINFLEX_BDRL = const LINFLEX_BDRL,
        EINVAL_NEG = const EINVAL,
        
        options(noreturn)
    );
}

/// int console_linflex_putc(int c, console_t *console);
///
/// Function to output a character over the console. It
/// returns the character printed on success or -EINVAL on error.
/// In : w0 - character to be printed
///      x1 - pointer to console_t struct
/// Out: w0 - printed character on success, < 0 on error.
/// Clobber list : x0 - x3, x15
#[no_mangle]
unsafe extern "C" fn console_linflex_putc(
    _c: i32,
    _console: &Console
) -> i32 {
    asm!(
        // Check if console pointer is NULL
        "cbz    x1, 2f",
        
        // Get base address from console struct
        "ldr    x1, [x1, {CONSOLE_T_BASE}]",

        // Jump to core putc function
        "b      console_linflex_core_putc",
        
        "2:", //putc_error
        "mov    x0, {EINVAL_NEG}",
        "ret",
        
        // Constants
        CONSOLE_T_BASE = const CONSOLE_T_BASE,
        EINVAL_NEG = const EINVAL,
        
        options(noreturn)
    );
}

/// void console_linflex_flush(console_t *console);
///
/// Function to wait for the TX FIFO to be cleared.
/// In : x0 - pointer to console_t struct
/// Clobber list : x0
#[no_mangle]
unsafe extern "C" fn console_linflex_flush(
    _console: &Console
) {
    asm!(
        // Get base address from console struct
        "ldr    x0, [x0, {CONSOLE_T_BASE}]",
        
        // Jump to core flush function
        "b      console_linflex_core_flush",
        
        // Note: The original has a redundant 'ret' after branch
        // which is never executed. I've removed it as it's unreachable.
        
        // Constants
        CONSOLE_T_BASE = const CONSOLE_T_BASE,
        
        options(noreturn)
    );
}

pub fn init()
{
    unsafe {
        console_linflex_register(UART_BASE, UART_CLOCK_HZ, UART_BAUD_RATE, &DEFAULTCONSOLE);
    }
}

/**
 * Send a single character to UART
 */
pub fn putc(c: u8) {
    unsafe {
        console_linflex_putc(c as i32, &DEFAULTCONSOLE);
    }
}

/**
 * Flush the transmit buffer
 */
pub fn flush() {
    unsafe {
        console_linflex_core_flush(UART_BASE);
    }
}

/**
 * Send a string to UART
 */
pub fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
    flush();
}

/**
 * Print a hexadecimal value
 */
pub fn print_hex(value: u32) {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    let mut buffer = [0; 11];  // "0x" + 8 hex digits + null terminator
    let mut value =value;
    buffer[0] = b'0';
    buffer[1] = b'x';
    
    for i in (2..10).rev() {
        buffer[i] = HEX_CHARS[(value & 0xF) as usize];
        value >>= 4;
    }
    
    puts(core::str::from_utf8(&buffer[0..10]).unwrap());
}

pub fn print_init_complete() {
    let msg = "\n\n\
              **********************************************\n\
              *                                            *\n\
              *  S32G3 FreeRTOS System Initialization      *\n\
              *  Successfully Completed                    *\n\
              *                                            *\n\
              *  Core 1 is now running FreeRTOS            *\n\
              *  Core 0 has returned to AT-F               *\n\
              *                                            *\n\
              **********************************************\n\n";
    
    puts(msg);
}

pub fn print_init_message(message: &str) {
    puts("[INIT] ");
    puts(message);
    puts("\n");
}

pub fn print_core_status(core_id: u32, status: &str) {
    let core_char = (b'0' + core_id as u8) as char;
    
    puts("Core ");
    putc(core_char as u8);
    puts(": ");
    puts(status);
    puts("\n");
}

// Implement formatting traits for UART output
struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}

// Format a string and print it via UART
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::uart::_print(format_args!($($arg)*)));
}

// Format a string with a newline and print it via UART
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Internal print function
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    UartWriter.write_fmt(args).unwrap();
}

// Format helper function that returns a String
pub fn format(args: fmt::Arguments) -> alloc::string::String {
    use core::fmt::Write;
    let mut output = alloc::string::String::new();
    output.write_fmt(args).unwrap();
    output
}