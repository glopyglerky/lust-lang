// expect: L0102
fn main() {
    let r: Result<String, FromUtf8Error> = String::from_utf8(vec![1]);
    println!("{:?}", r);
}
