#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rose::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rose::println;
use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    rose::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rose::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
