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

// 只读权限，却要写入
fn write_readonly_test() {
    extern "C" {
        fn srodata();
    }
    unsafe {
        let ptr = srodata as usize as *mut u8;
        *ptr = 0xab;
    }
}

// 不允许执行，非要执行
fn execute_unexecutable_test() {
    extern "C" {
        fn sbss();
    }
    unsafe {
        llvm_asm!("jr $0" :: "r"(sbss as usize) :: "volatile");
    }
}

// 找不到页表项
fn read_invalid_test() {
    println!("{}", unsafe { *(0x12345678 as usize as *const u8) });
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn run_main() -> ! {
    extern "C" {
        fn end();
    }
    sys_init();

    //frame_allocating_test();
    //unsafe {
    //    llvm_asm!("ebreak"::::"volatile");
    //}
    //write_readonly_test();
    //execute_unexecutable_test();
    //read_invalid_test();
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
