#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]

#[allow(unused_imports)]
#[macro_use]
extern crate os;

use os::init::sys_init;

use os::memory::{alloc_frame, dealloc_frame};

global_asm!(include_str!("boot/entry64.asm"));

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn run_main() -> ! {
    extern "C" {
        fn end();
    }
    sys_init();
    frame_allocating_test();
    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    }
    loop {}
}

fn frame_allocating_test() {
    println!("alloc {:x?}", alloc_frame());
    let f = alloc_frame();
    println!("alloc {:x?}", f);
    println!("alloc {:x?}", alloc_frame());
    println!("dealloc {:x?}", f);
    dealloc_frame(f.unwrap());
    println!("alloc {:x?}", alloc_frame());
    println!("alloc {:x?}", alloc_frame());
}