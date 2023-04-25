mod smmuv3;
mod smmu_queue;
mod ste;

pub fn init() {
    smmuv3::init();
}