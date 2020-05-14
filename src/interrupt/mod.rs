use riscv::register::{
    scause,//它会记录中断发生的原因，还会记录该中断是不是一个外部中断
    sepc,//它会记录触发中断的那条指令的地址
    stvec,//设置如何寻找 S 态中断处理程序的起始地址，保存了中断向量表基址 BASE，同时还有模式 MODE。
    sscratch //根据 sscratch 的值是否为 0 来判断是在 S 态产生的中断还是 U 态（用户态）产生的中断
};

use crate::context::StackFrame;

global_asm!(include_str!("trap.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            // 中断处理总入口
            fn __alltraps();
        }
        sscratch::write(0);
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
    println!("------------ init interrupt! -------------");
}

#[no_mangle]
fn trap_handler(sf:&mut StackFrame) {
    println!("trap handler");
    sf.sepc += 2;
}