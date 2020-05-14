use riscv::register::{sstatus::Sstatus, scause::Scause};

#[repr(C)]
pub struct StackFrame {
    pub reg: [usize; 32], //寄存器
    pub sstatus: Sstatus, //S 态控制状态寄存器。保存全局中断使能标志，以及许多其他的状态
    pub sepc: usize, //它会记录触发中断的那条指令的地址
    pub stval: usize, //它会记录一些中断处理所需要的辅助信息，比如取指、访存、缺页异常，它会把发生问题的目标地址记录下来，这样我们在中断处理程序中就知道处理目标了。
    pub scause: Scause, //它会记录中断发生的原因，还会记录该中断是不是一个外部中断
}