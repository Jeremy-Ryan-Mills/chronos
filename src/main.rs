
#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;


#[unsafe(no_mangle)] // Don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    /*
     * This function is the entry point when
     * the linker looks for a function named
     * _start
     */
    println!("Hello World{}", "!");
    loop {}
}

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
