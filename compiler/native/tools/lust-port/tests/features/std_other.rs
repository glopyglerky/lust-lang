// feature: std.other
// form: a std type path with no precise rule yet, std::string::FromUtf16Error
fn conv(u: &[u16]) -> Result<String, std::string::FromUtf16Error> {
    String::from_utf16(u)
}
fn main() {
    println!("{:?} {:?}", conv(&[65, 98]), conv(&[0xD800]).is_err());
}
