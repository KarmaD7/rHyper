#[allow(dead_code)]
use aarch64_cpu::asm;

#[inline]
pub fn wait_for_ints() {
  asm::wfi();
}