// expect: L0540
#[derive(Clone, Copy)]
enum E {
    A = 255,
}
fn main() {
    let x = E::A as u8;
    println!("{}", x + 1);
}
