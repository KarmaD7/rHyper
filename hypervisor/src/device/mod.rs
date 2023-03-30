pub mod gicv2;
pub mod pl011;

pub use pl011 as uart;
pub use gicv2 as intr;

pub use pl011::{console_getchar, console_putchar};
pub use gicv2::{pending_irq, handle_irq, inject_irq};


pub fn init_early() {
    pl011::init();
    gicv2::init();
}
