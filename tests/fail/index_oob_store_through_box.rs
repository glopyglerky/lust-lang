// expect: L0541
fn main() {
    let mut b = Box::new([1u8, 2]);
    b[0] = 3;
    b[5] = 3;
    println!("{}", b[0]);
}
