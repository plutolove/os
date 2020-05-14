use crate::interrupt;
pub fn sys_init() {
    interrupt::init();
    interrupt::timer::init();
}
