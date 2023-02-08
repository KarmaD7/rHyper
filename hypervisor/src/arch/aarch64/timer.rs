use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0};
use tock_registers::interfaces::Readable;

use crate::timer::NANOS_PER_SEC;

pub fn current_ticks() -> u64 {
  return CNTPCT_EL0.get();
}

pub fn ticks_to_nanos(ticks: u64) -> u64 {
  return ticks * NANOS_PER_SEC / CNTFRQ_EL0.get();
}