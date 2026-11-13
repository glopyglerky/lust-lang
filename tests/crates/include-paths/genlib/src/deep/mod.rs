// Includes in a nested module of a path dependency: the OUT_DIR form
// reads this crate's build script output, and the relative form in a
// macro body resolves against this file's directory.
macro_rules! load_deep {
    ($name:literal) => {
        include_str!(concat!($name, ".txt"))
    };
}

pub fn dep_out() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/dep.txt"))
}

pub fn dep_deep() -> &'static str {
    load_deep!("deep")
}
