// panic: called `Result::unwrap()` on an `Err` value: FromUtf8Error
fn main() {
    let s = String::from_utf8(vec![0x80]).unwrap();
    println!("{}", s);
}
