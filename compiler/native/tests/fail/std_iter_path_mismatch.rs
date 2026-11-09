// expect: L0110
// rustc E0308: `vec::IntoIter<u16>` is not `vec::IntoIter<u8>`.
fn bad() -> std::vec::IntoIter<u8> {
    vec![1u16].into_iter()
}

fn main() {
    println!("{}", bad().count());
}
