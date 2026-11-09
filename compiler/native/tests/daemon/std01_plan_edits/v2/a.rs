// A non-root file names an Option method: the splice grows.
pub fn go() {
    let x: Option<u8> = Some(3);
    println!("{}", x.unwrap_or(0));
}
