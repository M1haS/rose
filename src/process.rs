extern crate alloc;

use super::gdt;
use spin::RwLock;
use alloc::vec::Vec;
use x86_64::VirtAddr;
use lazy_static::lazy_static;
use super::interrupts::{INTERRUPT_CONTEXT_SIZE, Context};
use alloc::{boxed::Box, collections::vec_deque::VecDeque};

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 5;

struct Thread {
    kernel_stack:       Vec<u8>,
    user_stack:         Vec<u8>,
    kernel_stack_end:   u64, // This address goes in the TSS
    user_stack_end:     u64,
    context:            u64, // Address of Context on kernel stack
}

lazy_static! {
    static ref RUNNING_QUEUE: RwLock<VecDeque<Box<Thread>>> =
        RwLock::new(VecDeque::new());

    static ref CURRENT_THREAD: RwLock<Option<Box<Thread>>> =
        RwLock::new(None);
}

pub fn new_kthread(function: fn() -> ()) {
    use x86_64::instructions::interrupts;

    let new_thread = {
        let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE);
        let kernel_stack_end = (VirtAddr::from_ptr(kernel_stack.as_ptr())
                               + KERNEL_STACK_SIZE).as_u64();
        let user_stack = Vec::with_capacity(USER_STACK_SIZE);
        let user_stack_end = (VirtAddr::from_ptr(user_stack.as_ptr())
                              + USER_STACK_SIZE).as_u64() as usize;
        let context = kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64;

        Box::new(Thread {
            kernel_stack,
            user_stack,
            kernel_stack_end,
            user_stack_end: user_stack_end as u64,
            context})
    };

    let context = unsafe {&mut *(new_thread.context as *mut Context)};
    context.rip = function as usize; // Instruction pointer
    context.rsp = new_thread.user_stack_end as usize; // Stack pointer
    context.rflags = 0x200; // Interrupts enabled

    let (code_selector, data_selector) = gdt::get_kernel_segments();
    context.cs = code_selector.0 as usize;
    context.ss = data_selector.0 as usize;
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.write().push_back(new_thread);
    });
}

pub fn schedule_next(context_addr: usize) -> usize {
    let mut running_queue = RUNNING_QUEUE.write();
    let mut current_thread = CURRENT_THREAD.write();

    if let Some(mut thread) = current_thread.take() {
        // Save the location of the Context struct
        thread.context = context_addr as u64;
        // Put to the back of the queue
        running_queue.push_back(thread);
    }
    // Get the next thread in the queue
    *current_thread = running_queue.pop_front();
    match current_thread.as_ref() {
        Some(thread) => {
            // Set the kernel stack for the next interrupt
            gdt::set_interrupt_stack_table(
              gdt::TIMER_INTERRUPT_INDEX as usize,
              VirtAddr::new(thread.kernel_stack_end));
            // Point the stack to the new context
            thread.context as usize
          },
        None => 0  // Timer handler won't modify stack
    }
}