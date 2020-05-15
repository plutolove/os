use super::Tid;
use crate::interrupt::{disable_and_store, enable_and_wfi, restore};
use crate::process::thread_pool::ThreadPool;
use crate::process::Thread;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

#[derive(Clone)]
pub enum Status {
    Ready,
    Running(Tid),
    Sleeping,
    Exited(usize),
}

pub trait Scheduler {
    // 如果 tid 不存在，表明将一个新线程加入线程调度
    // 否则表明一个已有的线程要继续运行
    fn push(&mut self, tid: Tid);
    // 从若干可运行线程中选择一个运行
    fn pop(&mut self) -> Option<Tid>;
    // 时钟中断中，提醒调度算法当前线程又运行了一个 tick
    // 返回的 bool 表示调度算法认为当前线程是否需要被切换出去
    fn tick(&mut self) -> bool;
    // 告诉调度算法一个线程已经结束
    fn exit(&mut self, tid: Tid);
}

#[derive(Default)]
struct RRInfo {
    valid: bool,
    time: usize,
    prev: usize,
    next: usize,
}

pub struct RRScheduler {
    threads: Vec<RRInfo>,
    max_time: usize,
    current: usize,
}

impl RRScheduler {
    // 设置每个线程连续运行的最大 tick 数
    pub fn new(max_time_slice: usize) -> Self {
        let mut rr = RRScheduler {
            threads: Vec::default(),
            max_time: max_time_slice,
            current: 0,
        };
        rr.threads.push(RRInfo {
            valid: false,
            time: 0,
            prev: 0,
            next: 0,
        });
        rr
    }
}

impl Scheduler for RRScheduler {
    // 分为 1. 新线程 2. 时间片耗尽被切换出的线程 两种情况
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        if tid + 1 > self.threads.len() {
            self.threads.resize_with(tid + 1, Default::default);
        }

        if self.threads[tid].time == 0 {
            self.threads[tid].time = self.max_time;
        }

        let prev = self.threads[0].prev;
        self.threads[tid].valid = true;
        self.threads[prev].next = tid;
        self.threads[tid].prev = prev;
        self.threads[0].prev = tid;
        self.threads[tid].next = 0;
    }

    fn pop(&mut self) -> Option<Tid> {
        let ret = self.threads[0].next;
        if ret != 0 {
            let next = self.threads[ret].next;
            let prev = self.threads[ret].prev;
            self.threads[next].prev = prev;
            self.threads[prev].next = next;
            self.threads[ret].prev = 0;
            self.threads[ret].next = 0;
            self.threads[ret].valid = false;
            self.current = ret;
            Some(ret - 1)
        } else {
            None
        }
    }

    // 当前线程的可用时间片 -= 1
    fn tick(&mut self) -> bool {
        let tid = self.current;
        if tid != 0 {
            self.threads[tid].time -= 1;
            if self.threads[tid].time == 0 {
                return true;
            } else {
                return false;
            }
        }
        return true;
    }

    fn exit(&mut self, tid: Tid) {
        let tid = tid + 1;
        if self.current == tid {
            self.current = 0;
        }
    }
}

pub struct ProcessorInner {
    pool: Box<ThreadPool>,
    idle: Box<Thread>,
    current: Option<(Tid, Box<Thread>)>,
}

pub struct Processor {
    inner: UnsafeCell<Option<ProcessorInner>>,
}
unsafe impl Sync for Processor {}

impl Processor {
    pub const fn new() -> Processor {
        Processor {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn init(&self, idle: Box<Thread>, pool: Box<ThreadPool>) {
        unsafe {
            *self.inner.get() = Some(ProcessorInner {
                pool,
                idle,
                current: None,
            });
        }
    }

    fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized!")
    }
    // 通过线程池新增线程
    pub fn add_thread(&self, thread: Box<Thread>) {
        self.inner().pool.add(thread);
    }

    pub fn idle_main(&self) -> ! {
        let inner = self.inner();
        // 在 idle 线程刚进来时禁用异步中断
        disable_and_store();
        loop {
            // 如果从线程池中获取到一个可运行线程
            if let Some(thread) = inner.pool.acquire() {
                // 将自身的正在运行线程设置为刚刚获取到的线程
                inner.current = Some(thread);
                // 从正在运行的线程 idle 切换到刚刚获取到的线程
                println!(
                    "\n>>>> will switch_to thread {} in idle_main!",
                    inner.current.as_mut().unwrap().0
                );
                inner
                    .idle
                    .switch_to(&mut *inner.current.as_mut().unwrap().1);

                // 上个线程时间耗尽，切换回调度线程 idle
                println!("<<<< switch_back to idle in idle_main!");
                // 此时 current 还保存着上个线程
                let (tid, thread) = inner.current.take().unwrap();
                // 通知线程池这个线程需要将资源交还出去
                inner.pool.retrieve(tid, thread);
            }
            // 如果现在并无任何可运行线程
            else {
                // 打开异步中断，并等待异步中断的到来
                enable_and_wfi();
                // 异步中断处理返回后，关闭异步中断
                disable_and_store();
            }
        }
    }
    pub fn tick(&self) {
        let inner = self.inner();
        if !inner.current.is_none() {
            // 如果当前有在运行线程
            if inner.pool.tick() {
                // 如果返回true, 表示当前运行线程时间耗尽，需要被调度出去

                // 我们要进入 idle 线程了，因此必须关闭异步中断
                // 我们可没保证 switch_to 前后 sstatus 寄存器不变
                // 因此必须手动保存
                let flags = disable_and_store();

                // 切换到 idle 线程进行调度
                inner.current.as_mut().unwrap().1.switch_to(&mut inner.idle);

                // 之后某个时候又从 idle 线程切换回来
                // 恢复 sstatus 寄存器继续中断处理
                restore(flags);
            }
        }
    }
    pub fn exit(&self, code: usize) -> ! {
        // 由于要切换到 idle 线程，必须先关闭时钟中断
        disable_and_store();
        // 由于自己正在执行，可以通过这种方式获取自身的 tid
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;
        // 通知线程池这个线程退出啦！
        inner.pool.exit(tid);
        println!("thread {} exited, exit code = {}", tid, code);

        // 切换到 idle 线程决定下一个运行哪个线程
        inner.current.as_mut().unwrap().1.switch_to(&mut inner.idle);

        loop {}
    }

    pub fn run(&self) {
        // 运行，也就是从启动线程切换到调度线程 idle
        Thread::get_boot_thread().switch_to(&mut self.inner().idle);
    }
}
