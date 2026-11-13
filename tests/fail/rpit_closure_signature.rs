// expect: L0612
// rustc E0631: the closure takes u16, the bound wants `Fn(u8) -> u8`.
fn adder() -> impl Fn(u8) -> u8 {
    |x: u16| x as u8
}

fn main() {}
