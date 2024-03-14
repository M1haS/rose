#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rose::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use rose::{allocator, process, println, print};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::arch::asm;

extern crate alloc;

entry_point!(kernel_main);

fn kernel_thread_main() {
    // Launch another kernel thread
    process::new_kthread(test_kernel_fn2);

    loop {
        print!("<< thread 1 >>");
        x86_64::instructions::hlt();
    }
}

fn test_kernel_fn2() {
    loop {
        print!("<< thread 2 >>");
        x86_64::instructions::hlt();
    }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rose::memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    rose::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { rose::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    process::new_kthread(kernel_thread_main);
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