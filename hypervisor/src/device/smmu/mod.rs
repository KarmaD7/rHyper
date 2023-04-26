mod smmu_queue;
mod smmuv3;
mod ste;

pub fn init() {
    smmuv3::init();
}
