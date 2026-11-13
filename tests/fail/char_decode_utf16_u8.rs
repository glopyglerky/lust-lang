// expect: L0110
// rustc E0271: decode_utf16 takes u16 code units, not bytes.
fn main() {
    let n = char::decode_utf16(vec![b'a'].into_iter()).count();
    println!("{n}");
}
