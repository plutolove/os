pub mod timer;

use riscv::register::{
    scause::{Exception, Interrupt, Trap}, //它会记录中断发生的原因，还会记录该中断是不是一个外部中断
    // sepc,                                       它会记录触发中断的那条指令的地址
    sscratch, //根据 sscratch 的值是否为 0 来判断是在 S 态产生的中断还是 U 态（用户态）产生的中断
    sstatus,
    stvec, //设置如何寻找 S 态中断处理程序的起始地址，保存了中断向量表基址 BASE，同时还有模式 MODE。
};

use crate::context::StackFrame;
use timer::{clock_set_next_event, TICKS};
global_asm!(include_str!("trap.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            // 中断处理总入口
            fn __alltraps();
        }
        sscratch::write(0);
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
        sstatus::set_sie();
    }
    println!("------------ init interrupt! -------------");
}

#[no_mangle]
fn trap_handler(sf: &mut StackFrame) {
    match sf.scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut sf.sepc),
        Trap::Interrupt(Interrupt::SupervisorTimer) => timer_handler(),
        _ => panic!("undefined trap"),
    }
}

fn breakpoint(sepc: &mut usize) {
    println!("a breakpoint set @0x{:x}", sepc);
    *sepc += 2;
}

// s态时钟中断处理
fn timer_handler() {
    clock_set_next_event();
    unsafe {
        TICKS += 1;
        if TICKS == 100 {
            TICKS = 0;
            println!("* 100 ticks *");
        }
    }
}
