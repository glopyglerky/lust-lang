// String::from_utf8_lossy owns its text when a byte is invalid.
fn main() {
    let v = vec![104, 0xff];
    println!("{}", String::from_utf8_lossy(&v));
}
