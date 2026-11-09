// expect: L0510
// The fields of std's FromUtf8Error are private.
fn main() {
    if let Err(e) = String::from_utf8(vec![0xff]) {
        println!("{:?}", e.bytes);
    }
}
