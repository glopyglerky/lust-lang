pub mod inner {
#[macro_export]
macro_rules! twice { ($e:expr) => { $e * 5 }; }
pub use crate::twice;
pub fn base() -> i64 { 7 }
}
