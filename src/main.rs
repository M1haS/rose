#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rose::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use rose::println;
use x86_64::structures::paging::Page;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rose::memory::BootInfoFrameAllocator;

    println!("Hello World{}", "!");
    rose::init();

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rose::hlt_loop();   
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rose::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rose::test_panic_handler(info)
}