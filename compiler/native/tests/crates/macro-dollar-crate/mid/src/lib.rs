pub use base::{outer, sum_ones};

#[macro_export]
macro_rules! count {
    () => (0usize);
    ($x:expr $(, $rest:expr)*) => (1usize + $crate::count!($($rest),*));
}
