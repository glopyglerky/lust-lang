// Macro recursion to the deepest level rustc allows at the default
// recursion limit (128): the innermost r!() expands at depth 127.
macro_rules! r {
    () => { 0 };
    ($h:tt $($t:tt)*) => { 1 + r!($($t)*) };
}

fn main() {
    let n: i32 = r!(a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a);
    println!("{}", n);
}
