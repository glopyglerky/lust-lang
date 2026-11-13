// expect: L0001
// An emoji is not an XID identifier char.
fn main() {
    let 🦀 = 1;
    println!("{}", 🦀);
}
