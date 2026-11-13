// expect: L0801
// rustc: recursion limit reached while expanding `r!` (one level past the default 128).
macro_rules! r {
    () => { 0 };
    ($h:tt $($t:tt)*) => { 1 + r!($($t)*) };
}

fn main() {
    let n: i32 = r!(a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a);
    println!("{}", n);
}
