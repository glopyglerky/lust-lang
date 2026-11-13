// expect: L0307
// rustc E0515: a reference to the local array copy, not to the
// literal's static bytes.
fn bad() -> &'static [u8; 2] {
    let a = *b"ab";
    &a
}

fn main() {
    println!("{:?}", bad());
}
