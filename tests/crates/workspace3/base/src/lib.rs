pub mod inner;
pub use inner::{Kind, Thing};

#[macro_export]
macro_rules! base_add {
    ($e:expr) => {
        $e + $crate::inner::BASE
    };
}
