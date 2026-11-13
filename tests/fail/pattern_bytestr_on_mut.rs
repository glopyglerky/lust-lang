// expect: L0110
fn main() {
    let mut a = [97u8, 98];
    let x: &mut [u8] = &mut a;
    println!("{}", matches!(x, b"ab"));
}
