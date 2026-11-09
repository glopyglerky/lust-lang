// expect: L0801
// rustc: recursion limit reached while expanding. unreachable!("x") at depth 123 needs
// five more levels inside std (format_args! and friends), past the default limit.
macro_rules! r {
    () => { if false { unreachable!("x") } };
    ($h:tt $($t:tt)*) => { r!($($t)*) };
}

fn main() {
    r!(a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a);
}
