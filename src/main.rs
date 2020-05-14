#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]

#[warn(unreachable_code)]
#[allow(unused_imports)]

#[macro_use]
extern crate os;

use os::init::sys_init;

global_asm!(include_str!("boot/entry64.asm"));

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn run_main() -> ! {

    sys_init();
    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    }
    panic!("trap end of run main");
}