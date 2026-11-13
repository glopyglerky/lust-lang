// A file module that re-exports its macros with one braced `use`
// (jiff-core's macros.rs does this).
macro_rules! first {
    ($v:expr) => {
        match $v {
            Ok(v) => v,
            Err(_) => 0,
        }
    };
}
macro_rules! second {
    () => {
        2
    };
}
pub(crate) use {first, second};
