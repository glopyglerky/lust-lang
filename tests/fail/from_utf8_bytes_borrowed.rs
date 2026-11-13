// expect: L0311
// as_bytes borrows the error, so into_bytes cannot move it yet.
fn main() {
    if let Err(e) = String::from_utf8(vec![0xff]) {
        let b = e.as_bytes();
        let v = e.into_bytes();
        println!("{:?} {:?}", b, v);
    }
}
