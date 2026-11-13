// expect: L0306
// str::from_utf8 borrows its bytes.
fn main() {
    let s: &str;
    {
        let v = vec![104u8];
        s = match std::str::from_utf8(&v) {
            Ok(t) => t,
            Err(_) => "",
        };
    }
    println!("{}", s);
}
