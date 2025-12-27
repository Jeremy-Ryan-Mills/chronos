//! Serial output support.
//!
//! This module provides a simple, synchronized interface for printing text to
//! the first serial port (COM1). It is primarily intended for early boot
//! debugging and kernel logging, where VGA or more complex output facilities
//! may not yet be available.

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    /// Global handle to the first serial port (COM1, I/O port 0x3F8).
    ///
    /// Wrapped in a spinlock to allow safe shared access from different contexts,
    /// including interrupt handlers. The port is initialized once at startup.
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Low-level serial printing routine.
///
/// This function is not meant to be called directly. It is used by the
/// [`serial_print!`] and [`serial_println!`] macros.
///
/// Interrupts are temporarily disabled while holding the serial lock to avoid
/// deadlock if an interrupt handler attempts to write to the serial port while
/// it is already in use.
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints formatted text to the host through the serial interface.
///
/// This macro behaves like [`print!`], but sends its output over the serial
/// port instead of the VGA text buffer.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints formatted text to the host through the serial interface,
/// appending a newline.
///
/// This macro behaves like [`println!`], but sends its output over the serial
/// port instead of the VGA text buffer.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
