// expect: L0505
// The fields of std's Utf8Error are private (rustc E0451).
use std::str::Utf8Error;
fn main() {
    let u = Utf8Error { valid_up_to: 0, error_len: None };
    println!("{:?}", u);
}
