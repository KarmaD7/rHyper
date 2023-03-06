pub mod pl011;

pub use pl011 as uart;
pub(super) use pl011::init;
pub use pl011::{console_getchar, console_putchar};

pub fn init_early() {
    pl011::init();
}
