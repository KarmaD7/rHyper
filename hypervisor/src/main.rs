mod device;
mod mm;

fn main() {
    device::init_early();
    println!("Hello, world!");
}
