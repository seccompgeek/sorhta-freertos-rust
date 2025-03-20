use core::fmt;
use core::ptr::{read_volatile, write_volatile};
use crate::arch::s32g3::{
    UART_BASE,
    LINFLEX_LINCR1, LINFLEX_LINSR, LINFLEX_UARTCR, LINFLEX_UARTSR,
    LINFLEX_LINIBRR, LINFLEX_LINFBRR, LINFLEX_BDRL, LINFLEX_UARTPTO,
    LINCR1_INIT, LINCR1_MME, LINSR_LINS_MASK, LINSR_LINS_INITMODE,
    UARTCR_UART, UARTCR_WL0, UARTCR_PC0, UARTCR_PC1, UARTCR_TXEN,
    UARTCR_RXEN, UARTCR_TFBM, UARTCR_RFBM, UARTCR_ROSE, UARTCR_TFC,
    UARTSR_DTF, UART_CLOCK_HZ, UART_BAUD_RATE, LDIV_MULTIPLIER
};

/**
 * Calculate and set the baud rate generator registers
 */
fn linflex_set_brg(clock: u32, baud: u32) {
    unsafe {
        let linibrr = (UART_BASE + LINFLEX_LINIBRR) as *mut u32;
        let linfbrr = (UART_BASE + LINFLEX_LINFBRR) as *mut u32;
        let uartcr = (UART_BASE + LINFLEX_UARTCR) as *mut u32;
        let mut ldiv_mult = LDIV_MULTIPLIER;

        // Check if Reduced Oversampling is enabled
        let cr_val = read_volatile(uartcr);
        if cr_val & UARTCR_ROSE != 0 {
            // Extract OSR field if ROSE is set
            ldiv_mult = (cr_val >> 24) & 0xF;
        }

        // Calculate integer and fractional dividers
        let dividr = baud * ldiv_mult;
        let divisr = clock;
        
        let ibr = divisr / dividr;
        let mut fbr = ((divisr % dividr) * 16) / dividr;
        fbr &= 0xF;

        // Set the baud rate registers
        write_volatile(linibrr, ibr);
        write_volatile(linfbrr, fbr);
    }
}

/**
 * Initialize the LinFLEX UART for console output
 */
pub fn init() {
    unsafe {
        let lincr1 = (UART_BASE + LINFLEX_LINCR1) as *mut u32;
        let linsr = (UART_BASE + LINFLEX_LINSR) as *mut u32;
        let uartcr = (UART_BASE + LINFLEX_UARTCR) as *mut u32;
        let uartpto = (UART_BASE + LINFLEX_UARTPTO) as *mut u32;
        
        // Set master mode and init mode
        write_volatile(lincr1, LINCR1_INIT);
        write_volatile(lincr1, LINCR1_MME | LINCR1_INIT);
        
        // Wait for init mode entry
        while (read_volatile(linsr) & LINSR_LINS_MASK) != LINSR_LINS_INITMODE {
            // Wait
        }
        
        // Set UART bit
        write_volatile(uartcr, UARTCR_UART);
        
        // Set baud rate
        linflex_set_brg(UART_CLOCK_HZ, UART_BAUD_RATE);
        
        // Set preset timeout register value
        write_volatile(uartpto, 0xF);
        
        // 8-bit data, no parity, Tx/Rx enabled, UART mode, FIFO mode
        write_volatile(uartcr, UARTCR_PC1 | UARTCR_RXEN | UARTCR_TXEN | UARTCR_PC0 | 
                  UARTCR_WL0 | UARTCR_UART | UARTCR_RFBM | UARTCR_TFBM);
        
        // End init mode
        write_volatile(lincr1, read_volatile(lincr1) & !LINCR1_INIT);
    }
}

/**
 * Wait for the transmit buffer to be empty
 */
fn uart_wait_tx_complete() {
    unsafe {
        let uartcr = (UART_BASE + LINFLEX_UARTCR) as *mut u32;
        let uartsr = (UART_BASE + LINFLEX_UARTSR) as *mut u32;
        
        // Check if FIFO mode or buffer mode
        let is_fifo_mode = read_volatile(uartcr) & UARTCR_TFBM;
        
        if is_fifo_mode != 0 {
            // FIFO mode - wait for DTF flag to clear
            while read_volatile(uartsr) & UARTSR_DTF != 0 {
                // Wait
            }
        } else {
            // Buffer mode - wait for DTF flag to set, then clear it
            while read_volatile(uartsr) & UARTSR_DTF == 0 {
                // Wait
            }
            write_volatile(uartsr, UARTSR_DTF);  // Clear the flag in buffer mode
        }
    }
}

/**
 * Send a single character to UART
 */
pub fn putc(c: u8) {
    unsafe {
        let bdrl = (UART_BASE + LINFLEX_BDRL) as *mut u32;
        let uartcr = (UART_BASE + LINFLEX_UARTCR) as *mut u32;
        let uartsr = (UART_BASE + LINFLEX_UARTSR) as *mut u32;
        
        // If it's a newline, send carriage return first
        if c == b'\n' {
            putc(b'\r');
        }
        
        // Check if FIFO mode or buffer mode
        let is_fifo_mode = read_volatile(uartcr) & UARTCR_TFBM;
        
        if is_fifo_mode != 0 {
            // FIFO mode - wait for DTF flag to clear
            while read_volatile(uartsr) & UARTSR_DTF != 0 {
                // Wait
            }
        }
        
        // Write character to data register
        write_volatile(bdrl, c as u32);
        
        if is_fifo_mode == 0 {
            // Buffer mode - wait for DTF flag to set, then clear it
            while read_volatile(uartsr) & UARTSR_DTF == 0 {
                // Wait
            }
            write_volatile(uartsr, UARTSR_DTF);  // Clear the flag in buffer mode
        }
    }
}

/**
 * Flush the transmit buffer
 */
pub fn flush() {
    unsafe {
        let uartcr = (UART_BASE + LINFLEX_UARTCR) as *mut u32;
        
        // Check if FIFO mode or buffer mode
        let is_fifo_mode = read_volatile(uartcr) & UARTCR_TFBM;
        
        if is_fifo_mode != 0 {
            // In FIFO mode, wait until TFC counter is zero
            while (read_volatile(uartcr) & UARTCR_TFC) != 0 {
                // Wait
            }
        } else {
            // In buffer mode, just ensure the last character was sent
            uart_wait_tx_complete();
        }
    }
}

/**
 * Send a string to UART
 */
pub fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
    flush();  // Ensure the output is flushed
}

/**
 * Print a hexadecimal value
 */
pub fn print_hex(value: u32) {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    let mut buffer = [0; 11];  // "0x" + 8 hex digits + null terminator
    
    buffer[0] = b'0';
    buffer[1] = b'x';
    
    for i in (2..10).rev() {
        buffer[i] = HEX_CHARS[(value & 0xF) as usize];
        value >> 4;
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