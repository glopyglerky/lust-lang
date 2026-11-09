// expect: L0612
// rustc E0277: u8 is not a writer.
fn main() {
    let mut r: &[u8] = b"ab";
    println!("{}", std::io::copy(&mut &mut r, &mut 5u8).unwrap());
}
