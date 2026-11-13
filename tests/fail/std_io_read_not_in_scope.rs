// expect: L0104
// read_to_end on a slice needs io::Read in scope.
fn main() {
    let mut s: &[u8] = b"xy";
    let mut v: Vec<u8> = Vec::new();
    let _ = s.read_to_end(&mut v);
    println!("{:?}", v);
}
