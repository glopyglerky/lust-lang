// expect: L0733
// rustc E0621: `a as &'a u8` needs `a` to live for `'a`, which its elided lifetime does not say.
fn h<'a>(a: &u8) -> &'a u8 {
    a as &'a u8
}
fn main() {
    let x = 1u8;
    println!("{}", h(&x));
}
