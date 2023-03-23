#![feature(global_asm)]

use core::fmt::{self, Write};

const UART_BASE_ADDR: usize = 0x1000_0000;

// Initialize the UART
fn uart_init() {
    // Enable the UART clock
    unsafe {
        let plic_base_addr = 0x0c00_0000 as *mut u32;
        let plic_enable_offset = 0x0020 / 4;
        let plic_uart0_enable = 1 << 10;
        plic_base_addr
            .add(plic_enable_offset)
            .write_volatile(plic_uart0_enable);
    }

    // Set the UART baud rate and enable the transmitter and receiver
    unsafe {
        let uart_ptr = UART_BASE_ADDR as *mut u32;
        let baud_rate = 115200;
        let divisor = 50000000 / baud_rate;
        uart_ptr.add(0).write_volatile(0); // Disable UART
        uart_ptr.add(1).write_volatile(0); // Disable interrupts
        uart_ptr.add(3).write_volatile(0x80); // Enable divisor mode
        uart_ptr.add(0).write_volatile(divisor & 0xff); // Set low byte of divisor
        uart_ptr.add(1).write_volatile((divisor >> 8) & 0xff); // Set high byte of divisor
        uart_ptr.add(3).write_volatile(3); // 8-bit data, no parity, 1 stop bit
        uart_ptr.add(5).write_volatile(0x03); // Enable transmitter and receiver
    }
}

fn uart_write_byte(byte: u8) {
    unsafe {
        let uart_ptr = UART_BASE_ADDR as *mut u8;
        *uart_ptr = byte;
    }
}

fn uart_write_string(s: &str) {
    for byte in s.bytes() {
        uart_write_byte(byte);
    }
}

struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart_write_string(s);
        Ok(())
    }
}

macro_rules! print {
    ($($arg:tt)*) => {{
        let mut writer = UartWriter {};
        uart_init();
        write!(writer, $($arg)*).unwrap();
    }};
}

macro_rules! println {
    () => {{
        print!("\n");
    }};
    ($($arg:tt)*) => {{
        print!("{}\n", format_args!($($arg)*));
    }};
}

// Print the OS name and version
pub fn print_os() {
    println!("Rust OS v1.0");
}

/*
// Entry point of the OS
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Print the OS name and version
    print_os();

    // Loop forever
    loop {}
} */
