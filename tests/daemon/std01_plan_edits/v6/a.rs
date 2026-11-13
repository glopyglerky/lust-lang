// An error in a file that also names std source.
pub fn go() {
    let x: Option<u8> = Some(3);
    let y: u8 = x.unwrap_or("no");
    println!("{}", y);
}
