#![no_std]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]

#[macro_use]
pub mod io;

pub mod init;

pub mod memory;

mod context;

mod consts;
mod interrupt;
mod lang_items;
mod process;
mod sbi;

extern crate alloc;
