use crate::consts::*;
use crate::interrupt;
use crate::memory;
use crate::process;

pub fn sys_init() {
    extern "C" {
        fn end();
    }
    interrupt::init();
    interrupt::timer::init();
    memory::init(
        ((end as usize - KERNEL_BEGIN_VADDR + KERNEL_BEGIN_PADDR) >> 12) + 1,
        PHYSICAL_MEMORY_END >> 12,
    );
    process::init();
    process::run();
}
