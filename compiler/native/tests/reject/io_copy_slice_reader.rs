// expect: L0641
// rustc accepts this; std copies an in-memory `&[u8]` reader with one
// write_all, which lust does not model, so it rejects.
fn main() {
    let mut r: &[u8] = b"ab";
    let mut w = std::io::sink();
    println!("{}", std::io::copy(&mut r, &mut w).unwrap());
}
