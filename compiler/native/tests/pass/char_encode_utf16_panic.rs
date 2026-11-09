// `char::encode_utf16` into a buffer that is too short panics
// as core does (tf-str).
fn main() {
    let mut buf = [0u16; 1];
    let r = '\u{1F600}'.encode_utf16(&mut buf);
    println!("{:?}", r);
}
