// A macro in a nested module file: the include path is built by
// concat! from a literal fragment and resolves relative to this file.
macro_rules! load_lib {
    ($name:literal) => {
        include_str!(concat!("../data/", $name))
    };
}

pub fn lib_text() -> &'static str {
    load_lib!("lib.txt")
}

pub fn lib_bytes_len() -> usize {
    include_bytes!("../data/lib.txt").len()
}
