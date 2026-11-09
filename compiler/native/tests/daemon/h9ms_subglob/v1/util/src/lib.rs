pub mod inner {
#[macro_export]
macro_rules! twice { ($e:expr) => { $e * 2 }; }
pub use crate::twice;
pub fn base() -> i64 { 7 }
}
