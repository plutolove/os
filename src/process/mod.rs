pub mod scheduler;
pub mod thread_pool;

pub type Tid = usize;

use crate::alloc::alloc::{alloc, dealloc, Layout};
use crate::alloc::boxed::Box;
use crate::consts::*;
use crate::context::Context;
use riscv::register::satp;

pub struct Thread {
    pub context: Context,
    pub kstack: KernelStack,
}

impl Thread {
    pub fn switch_to(&mut self, target: &mut Thread) {
        unsafe {
            self.context.switch(&mut target.context);
        }
    }

    pub fn new_kernel(entry: usize) -> Box<Thread> {
        unsafe {
            let kstack_ = KernelStack::new();
            Box::new(Thread {
                // 内核线程共享内核资源，因此用目前的 satp 即可
                context: Context::new_kernel_thread(entry, kstack_.top(), satp::read().bits()),
                kstack: kstack_,
            })
        }
    }
    // 为线程传入初始参数
    pub fn append_initial_arguments(&self, args: [usize; 3]) {
        unsafe {
            self.context.append_initial_arguments(args);
        }
    }

    pub fn get_boot_thread() -> Box<Thread> {
        Box::new(Thread {
            context: Context::null(),
            kstack: KernelStack::new_empty(),
        })
    }
}

pub struct KernelStack(usize);
impl KernelStack {
    pub fn new() -> Self {
        let bottom = unsafe {
            alloc(Layout::from_size_align(KERNEL_STACK_SIZE, KERNEL_STACK_SIZE).unwrap()) as usize
        };
        KernelStack(bottom)
    }
    pub fn new_empty() -> Self {
        KernelStack(0)
    }
    pub fn top(&self) -> usize {
        self.0 + KERNEL_STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                dealloc(
                    self.0 as _,
                    Layout::from_size_align(KERNEL_STACK_SIZE, KERNEL_STACK_SIZE).unwrap(),
                );
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn temp_thread(from_thread: &mut Thread, current_thread: &mut Thread) {
    println!("Hello world! swatch to from thread");
    current_thread.switch_to(from_thread);
}

use scheduler::Processor;
static CPU: Processor = Processor::new();

pub fn exit(code: usize) {
    CPU.exit(code);
}

pub fn run() {
    CPU.run();
}

use scheduler::RRScheduler;
use thread_pool::ThreadPool;
pub fn init() {
    // 使用 Round Robin Scheduler
    let scheduler = RRScheduler::new(2);
    // 新建线程池
    let thread_pool = ThreadPool::new(100, Box::new(scheduler));
    // 新建内核线程 idle ，其入口为 Processor::idle_main
    let idle = Thread::new_kernel(Processor::idle_main as usize);
    // 我们需要传入 CPU 的地址作为参数
    idle.append_initial_arguments([&CPU as *const Processor as usize, 0, 0]);
    // 初始化 CPU
    CPU.init(idle, Box::new(thread_pool));

    // 依次新建 5 个内核线程并加入调度单元
    for i in 0..5 {
        CPU.add_thread({
            let thread = Thread::new_kernel(hello_thread as usize);
            // 传入一个编号作为参数
            thread.append_initial_arguments([i, 0, 0]);
            thread
        });
    }
    println!("++++ setup process!   ++++");
}

#[no_mangle]
pub extern "C" fn hello_thread(arg: usize) -> ! {
    println!("begin of thread {}", arg);
    println!("thread {} is running", arg);
    println!("end  of thread {}", arg);
    // 通知 CPU 自身已经退出
    CPU.exit(0);
    loop {}
}
