// A parenthesized base indexes at the `[` before the index; a `(` that a
// macro writes around its argument reports at the macro call.
// panic: attempt to add with overflow
macro_rules! bump {
    ($x:expr) => {
        ($x).checked_add(1).unwrap() + 255
    };
}
fn main() {
    let n = std::env::args().count();
    let v = vec![1u8, 2, 3];
    println!("{}", (&v)[n]);
    println!("{}", (v)[n - 1]);
    let r = (bump!(n as u8 - 1));
    println!("{}", r);
}
