// SMMUv3 stage-2 translation is not supported in qemu.

mod smmu_queue;
mod smmuv3;
mod ste;

pub fn init() {
    smmuv3::init();
}
