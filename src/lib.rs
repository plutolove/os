#![no_std]
#![feature(llvm_asm)]
#![feature(global_asm)]

#[macro_use]
mod io;

mod init;
mod lang_items;
mod sbi;