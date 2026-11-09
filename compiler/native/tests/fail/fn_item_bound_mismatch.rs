// expect: L0110
// rustc E0631: `double` takes u16, the bound wants `Fn(u8) -> u8`.
fn double(x: u16) -> u16 {
    x * 2
}

fn apply<F: Fn(u8) -> u8>(f: F) -> u8 {
    f(4)
}

fn main() {
    println!("{}", apply(double));
}
