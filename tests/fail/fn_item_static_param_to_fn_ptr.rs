// expect: L0110
// rustc E0308: `g` needs a `'static` borrow; `fn(&u8) -> u8` accepts any lifetime.
fn take(f: fn(&u8) -> u8) -> u8 {
    let x = 5u8;
    f(&x)
}

fn g(x: &'static u8) -> u8 {
    *x
}

fn main() {
    println!("{}", take(g));
}
