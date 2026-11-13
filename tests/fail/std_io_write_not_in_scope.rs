// expect: L0104
// write! into a Vec<u8> needs io::Write in scope.
fn main() {
    let mut v: Vec<u8> = Vec::new();
    let _ = write!(v, "x");
    println!("{:?}", v);
}
