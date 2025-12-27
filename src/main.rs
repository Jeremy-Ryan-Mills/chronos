#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(chronos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use chronos::println;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    chronos::init();

    // For testing
    #[cfg(test)]
    test_main();

    println!("It didnt crash yay");
    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// This function is called on panic while testing.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    chronos::test_panic_handler(info)
}
