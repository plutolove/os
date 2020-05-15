use core::mem::zeroed;
use riscv::register::sstatus;
use riscv::register::{scause::Scause, sstatus::Sstatus};

#[repr(C)]
pub struct StackFrame {
    pub reg: [usize; 32], //寄存器
    pub sstatus: Sstatus, //S 态控制状态寄存器。保存全局中断使能标志，以及许多其他的状态
    pub sepc: usize,      //它会记录触发中断的那条指令的地址
    pub stval: usize, //它会记录一些中断处理所需要的辅助信息，比如取指、访存、缺页异常，它会把发生问题的目标地址记录下来，这样我们在中断处理程序中就知道处理目标了。
    pub scause: Scause, //它会记录中断发生的原因，还会记录该中断是不是一个外部中断
}

#[repr(C)]
pub struct ContextContent {
    pub ra: usize,
    satp: usize,
    s: [usize; 12],
    sf: StackFrame,
}
extern "C" {
    fn __trapret();
}
impl ContextContent {
    // 为一个新内核线程构造栈上的初始状态信息
    // 其入口点地址为 entry ，其内核栈栈顶地址为 kstack_top ，其页表为 satp
    fn new_kernel_thread(entry: usize, kstack_top: usize, satp: usize) -> ContextContent {
        let mut content = ContextContent {
            ra: __trapret as usize,
            satp,
            s: [0; 12],
            sf: {
                let mut sf: StackFrame = unsafe { zeroed() };
                sf.reg[2] = kstack_top;
                sf.sepc = entry;
                sf.sstatus = sstatus::read();
                sf.sstatus.set_spp(sstatus::SPP::Supervisor);
                sf.sstatus.set_spie(true);
                sf.sstatus.set_sie(false);
                sf
            },
        };
        content
    }

    unsafe fn push_at(self, stack_top: usize) -> Context {
        let ptr = (stack_top as *mut ContextContent).sub(1);
        *ptr = self;
        Context {
            content_addr: ptr as usize,
        }
    }
}

#[repr(C)]
pub struct Context {
    pub content_addr: usize,
}

impl Context {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, target: &mut Context) {
        llvm_asm!(include_str!("../process/switch.asm") :::: "volatile");
    }

    pub unsafe fn new_kernel_thread(entry: usize, kstack_top: usize, satp: usize) -> Context {
        ContextContent::new_kernel_thread(entry, kstack_top, satp).push_at(kstack_top)
    }
    pub unsafe fn append_initial_arguments(&self, args: [usize; 3]) {
        let contextContent = &mut *(self.content_addr as *mut ContextContent);
        contextContent.sf.reg[10] = args[0];
        contextContent.sf.reg[11] = args[1];
        contextContent.sf.reg[12] = args[2];
    }

    pub fn null() -> Context {
        Context { content_addr: 0 }
    }
}
