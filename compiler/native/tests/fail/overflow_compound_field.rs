// expect: L0540
fn main() {
    let mut a: (u8, u8) = (3, 0);
    a.0 -= 4;
    println!("ran");
}
