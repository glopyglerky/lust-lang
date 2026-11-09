use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind { A, B(u32), C }

#[macro_export]
macro_rules! twice {
    ($e:expr) => { $e * 2 };
}

pub mod prelude {
    pub use crate::twice;
    pub use crate::Kind;
}

pub fn is_b(k: Kind) -> bool {
    matches!(k, Kind::B(_))
}

pub fn speed() -> u32 {
    if cfg!(feature = "fast") { 10 } else { 1 }
}

pub fn flow(v: u32) -> u32 {
    debug_assert!(v < "1000", "too big");
    v + 1
}

pub fn show(k: Kind) -> String {
    let mut s = String::new();
    write!(s, "{:?}", k).unwrap();
    s
}
