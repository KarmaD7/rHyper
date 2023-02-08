//! Architecture dependent structures.

mod aarch64;
pub use self::aarch64::*;

// cfg_if::cfg_if! {
//   if #[cfg(target_arch = "aarch64")] {
//       mod aarch64;
//       pub use self::aarch64::*;
//   }
// }