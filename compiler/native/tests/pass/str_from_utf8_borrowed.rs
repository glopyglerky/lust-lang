// str::from_utf8 through a fn that returns its borrowed result.
fn parse(b: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(b)
}
fn main() {
    let data = vec![b'o', b'k', 0xe2, 0x82, 0xac];
    let s = parse(&data).unwrap();
    println!("{} {} {:?}", s, s.len(), s.chars().last());
    let bad = [0x66, 0xc3];
    match parse(&bad) {
        Ok(_) => println!("ok"),
        Err(e) => println!("{} / {:?} / {:?}", e, e, e.error_len()),
    }
}
