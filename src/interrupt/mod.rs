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

fn page_fault(tf: &mut StackFrame) {
    println!(
        "{:?} va = {:#x} instruction = {:#x}",
        tf.scause.cause(),
        tf.stval,
        tf.sepc
    );
    panic!("page fault!");
}

#[no_mangle]
fn trap_handler(sf: &mut StackFrame) {
    match sf.scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut sf.sepc),
        Trap::Interrupt(Interrupt::SupervisorTimer) => timer_handler(),
        Trap::Exception(Exception::InstructionPageFault) => page_fault(sf),
        Trap::Exception(Exception::LoadPageFault) => page_fault(sf),
        Trap::Exception(Exception::StorePageFault) => page_fault(sf),
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

#[inline(always)]
pub fn disable_and_store() -> usize {
    let sstatus: usize;
    unsafe {
        // clear sstatus 的 SIE 标志位禁用异步中断
        // 返回 clear 之前的 sstatus 状态
        llvm_asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
    }
    sstatus
}

#[inline(always)]
pub fn restore(flags: usize) {
    unsafe {
        // 将 sstatus 设置为 flags 的值
        llvm_asm!("csrs sstatus, $0" :: "r"(flags) :: "volatile");
    }
}

#[inline(always)]
pub fn enable_and_wfi() {
    unsafe {
        // set sstatus 的 SIE 标志位启用异步中断
        // 并通过 wfi 指令等待下一次异步中断的到来
        llvm_asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
    }
}
