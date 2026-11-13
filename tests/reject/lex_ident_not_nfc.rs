// expect: L0002
// rustc treats the two spellings of this name as one name (NFC).
// Lust rejects the decomposed spelling (e + U+0301) on purpose.
fn main() {
    let café = 11;
    println!("{}", café);
}
