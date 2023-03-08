pub mod pl011;
pub mod gicv2;

pub use pl011 as uart;
pub use pl011::{console_getchar, console_putchar};

pub fn init_early() {
    pl011::init();
}
