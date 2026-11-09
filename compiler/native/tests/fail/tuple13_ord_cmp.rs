// expect: L0104
// rustc E0599: core implements Ord for tuples of at most 12 elements, so a
// 13-tuple has no `cmp` method.
fn main() {
    let t = (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8);
    println!("{:?}", t.cmp(&t));
}
