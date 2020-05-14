use crate::sbi::set_timer;
use riscv::register::{sie, time};

pub static mut TICKS: usize = 0;
static TIMEBASE: u64 = 100000;

fn get_cycle() -> u64 {
    time::read() as u64
}

pub fn clock_set_next_event() {
    set_timer(get_cycle() + TIMEBASE);
}

pub fn init() {
    unsafe {
        // 初始化时钟中断触发次数
        TICKS = 0;
        // 设置 sie 的 TI 使能 STIE 位
        sie::set_stimer();
    }
    // 硬件机制问题我们不能直接设置时钟中断触发间隔
    // 只能当每一次时钟中断触发时
    // 设置下一次时钟中断的触发时间
    // 设置为当前时间加上 TIMEBASE
    // 这次调用用来预处理
    clock_set_next_event();
    println!("--------- init timer -------------");
}
