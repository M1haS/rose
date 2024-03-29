use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{
    GlobalDescriptorTable, Descriptor, SegmentSelector
};
use core::{char::DecodeUtf16, ptr::addr_of};

pub const DOUBLE_FAULT_IST_INDEX:             u16 = 0;
pub const PAGE_FAULT_IST_INDEX:               u16 = 0;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 0;
pub const TIMER_INTERRUPT_INDEX:              u16 = 1;

lazy_static! {
    static ref TSS: Mutex<TaskStateSegment> = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss.interrupt_stack_table[TIMER_INTERRUPT_INDEX as usize] =
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize];

        Mutex::new(tss)
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(
            unsafe { tss_reference() }
        ));

        (gdt, Selectors { code_selector, data_selector, tss_selector })
    };
}

unsafe fn tss_reference() -> &'static TaskStateSegment {
    let tss_ptr = &*TSS.lock() as *const TaskStateSegment;
    & *tss_ptr
}

pub fn set_interrupt_stack_table(index: usize, stack_end: VirtAddr) {
    TSS.lock().interrupt_stack_table[index] = stack_end;
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
  (GDT.1.code_selector, GDT.1.data_selector)
}