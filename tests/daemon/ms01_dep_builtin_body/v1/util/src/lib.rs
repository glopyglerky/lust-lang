// A dependency with a macro of its own name, and none under a builtin's
// name: bodies of the main crate that call builtin macros patch.
#[macro_export]
macro_rules! twice {
    ($e:expr) => {
        $e * 2
    };
}

pub fn helper() -> i64 {
    2
}
