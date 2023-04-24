pub mod gicv2;
pub mod pl011;
pub mod virtio;
pub mod smmu;

pub use gicv2 as intr;
pub use pl011 as uart;
pub use smmu as iommu;

pub use gicv2::{handle_irq, inject_irq, pending_irq};
pub use pl011::{console_getchar, console_putchar};

pub fn init_early() {
    pl011::init();
}

pub fn init() {
    gicv2::init();
    // smmu::init();
}
