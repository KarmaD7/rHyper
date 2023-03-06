mod trap;

pub mod instructions;
pub mod timer;

pub fn init() {
    trap::init();
}
