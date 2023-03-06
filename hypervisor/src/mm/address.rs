use super::PAGE_SIZE;

pub type PhysAddr = usize;
pub type VirtAddr = usize;

pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr
}

pub const fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr
}

pub const fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    (addr & (PAGE_SIZE - 1)) == 0
}
