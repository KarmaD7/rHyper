use core::time::Duration;

pub type TimeValue = Duration;

pub fn current_time() -> TimeValue {
  TimeValue::new(0, 0)
}