mod frame_allocator;

pub const KERNEL_BEGIN_PADDR: usize = 0x80200000;
pub const KERNEL_BEGIN_VADDR: usize = 0x80200000;
pub const PHYSICAL_MEMORY_END: usize = 0x88000000;
pub const MAX_PHYSICAL_PAGES: usize = PHYSICAL_MEMORY_END >> 12;

use frame_allocator::SEG_FRAME_ALLOC;
use riscv::addr::{VirtAddr, PhysAddr, Page, Frame};

pub fn init(l: usize, r: usize) {
    SEG_FRAME_ALLOC.lock().init(l, r);
    println!("--------- init memory -----------");
}

pub fn alloc_frame() -> Option<Frame> {
    Some(Frame::of_ppn(SEG_FRAME_ALLOC.lock().alloc()))
}

pub fn dealloc_frame(f: Frame) {
    SEG_FRAME_ALLOC.lock().dealloc(f.number());
}