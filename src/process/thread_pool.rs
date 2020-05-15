use super::scheduler::Status;
use super::Thread;
use super::Tid;
use crate::process::scheduler::Scheduler;
use alloc::boxed::Box;
use alloc::vec::Vec;

struct Task {
    status: Status,
    thread: Option<Box<Thread>>,
}

pub struct ThreadPool {
    threads: Vec<Option<Task>>,
    scheduler: Box<dyn Scheduler>,
}

impl ThreadPool {
    pub fn new(size: usize, scheduler: Box<dyn Scheduler>) -> ThreadPool {
        ThreadPool {
            threads: {
                let mut v = Vec::new();
                v.resize_with(size, Default::default);
                v
            },
            scheduler,
        }
    }

    fn alloc_tid(&self) -> Tid {
        for (i, task) in self.threads.iter().enumerate() {
            if task.is_none() {
                return i;
            }
        }
        panic!("no tid to alloc");
    }

    pub fn add(&mut self, _thread: Box<Thread>) {
        let tid = self.alloc_tid();
        self.threads[tid] = Some(Task {
            status: Status::Ready,
            thread: Some(_thread),
        });
        self.scheduler.push(tid);
    }

    pub fn acquire(&mut self) -> Option<(Tid, Box<Thread>)> {
        if let Some(tid) = self.scheduler.pop() {
            let mut task = self.threads[tid].as_mut().expect("thread not exist!");
            task.status = Status::Running(tid);
            return Some((tid, task.thread.take().expect("thread not exist!")));
        } else {
            return None;
        }
    }

    pub fn retrieve(&mut self, tid: Tid, thread: Box<Thread>) {
        // 线程池位置为空，表明这个线程刚刚通过 exit 退出
        if self.threads[tid].is_none() {
            // 不需要 CPU 资源了，退出
            return;
        }
        // 获取并修改线程池对应位置的信息
        let mut thread_info = self.threads[tid].as_mut().expect("thread not exist!");
        thread_info.thread = Some(thread);
        // 此时状态可能是 Status::Sleeping(线程可能会自动放弃 CPU 资源，进入睡眠状态),
        // 直到被唤醒之前都不必给它分配。
        // 而如果此时状态是Running,就说明只是单纯的耗尽了这次分配CPU资源,但还要占用CPU资源继续执行。
        if let Status::Running(_) = thread_info.status {
            // Running -> Ready
            thread_info.status = Status::Ready;
            // 通知线程池继续给此线程分配资源
            self.scheduler.push(tid);
        }
    }

    // Scheduler 的简单包装：时钟中断时查看当前所运行线程是否要切换出去
    pub fn tick(&mut self) -> bool {
        let ret = self.scheduler.tick();
        ret
    }
    // 这个线程已经退出了，线程状态 Running -> Exited
    pub fn exit(&mut self, tid: Tid) {
        // 清空线程池对应位置
        self.threads[tid] = None;
        // 通知调度器
        self.scheduler.exit(tid);
    }
}
