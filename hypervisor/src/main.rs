#![no_std]
#![no_main]
#![feature(asm_const, naked_functions)]
#![feature(panic_info_message, alloc_error_handler)]

#[macro_use]
extern crate log;

#[macro_use]
mod logging;

mod arch;
mod device;
mod mm;
mod config;
mod timer;

#[cfg(not(test))]
mod lang_items;

use core::sync::atomic::{AtomicBool, Ordering};

static INIT_OK: AtomicBool = AtomicBool::new(false);

const LOGO: &str = r"

    RRRRRR  VV     VV MM    MM
    RR   RR VV     VV MMM  MMM
    RRRRRR   VV   VV  MM MM MM
    RR  RR    VV VV   MM    MM
    RR   RR    VVV    MM    MM
     ___    ____    ___    ___
    |__ \  / __ \  |__ \  |__ \
    __/ / / / / /  __/ /  __/ /
   / __/ / /_/ /  / __/  / __/
  /____/ \____/  /____/ /____/
";

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize).fill(0)
    }
}

pub fn init_ok() -> bool {
    INIT_OK.load(Ordering::SeqCst)
}

fn main() {
    clear_bss();
    device::init_early();
    println!("{}", LOGO);
    println!("Hello, world!");
    arch::instructions::wait_for_ints();
}