// expect: L0801
// rustc: recursion limit reached while expanding `$crate::format_args_nl!`. println! at
// depth 127 expands format_args_nl! at depth 128, past the default limit.
macro_rules! r {
    () => { println!("x") };
    ($h:tt $($t:tt)*) => { r!($($t)*) };
}

fn main() {
    r!(a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a);
}
