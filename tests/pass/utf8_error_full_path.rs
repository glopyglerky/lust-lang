// Utf8Error by its full std path.
fn check(e: &std::str::Utf8Error) -> usize {
    e.valid_up_to()
}
fn main() {
    let b = [97u8, 98, 0xff];
    let r = std::str::from_utf8(&b);
    if let Err(e) = r {
        println!("{} {:?}", check(&e), e.error_len());
    }
}
