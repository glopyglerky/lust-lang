// expect: L0110
// rustc E0308: `iter::once` returns `iter::Once<T>`, not `option::IntoIter<T>`.
fn main() {
    let x: std::option::IntoIter<u8> = std::iter::once(1);
    let _ = x;
}
