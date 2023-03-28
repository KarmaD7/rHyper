#![no_std]
#![no_main]
#![feature(asm_const, naked_functions)]
#![feature(panic_info_message, alloc_error_handler)]

#[macro_use]
extern crate log;

extern crate alloc;
#[macro_use]
mod logging;

mod arch;
mod config;
mod device;
mod hv;
mod mm;
mod platform;
mod timer;

mod utils;

#[cfg(not(test))]
mod lang_items;

use core::sync::atomic::{AtomicBool, Ordering};

use arch::instructions::wait_for_ints;
use device::console_putchar;
use spin::Mutex;

use crate::platform::mp::start_secondary_cpus;

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

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn boot_stack();
    fn boot_stack_top();
    fn ekernel();
}

fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0)
    }
}

pub fn init_ok() -> bool {
    INIT_OK.load(Ordering::SeqCst)
}

#[no_mangle]
fn rust_main(cpu_id: usize) {
    clear_bss();
    device::init_early();
    println!("{}", LOGO);
    println!("primary cpu id: {}.", cpu_id);
    println!(
        "boot stack: 0x{:x}, boot stack top: 0x{:x}",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(
        "\
        arch = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("ARCH").unwrap_or(""),
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
    );

    mm::init_heap_early();
    logging::init();
    info!("Logging is enabled.");

    mm::init();
    INIT_OK.store(true, Ordering::SeqCst);
    info!("Initialization completed.\n");
    start_secondary_cpus(cpu_id);
    hv::run(cpu_id);
    // arch::instructions::wait_for_ints();
}

#[no_mangle]
fn rust_main_secondary(cpu_id: usize) {
    // todo
    info!("Hello World from cpu {}", cpu_id);
    hv::run(cpu_id);
    // console_putchar('z' as u8);
    // console_putchar('b' as u8);
    // console_putchar('d' as u8);
    // info!("CPU {} initialized.", cpu_id);
    // info!("CPU {} initialized.", cpu_id);
    // wait_for_ints();
}

// fn test_output(val: &str) {
//     static LOCK: Mutex<()> = Mutex::new(());
//     LOCK.lock();
//     for s in val.chars() {
//         console_putchar(s as u8);
//     }
// }
