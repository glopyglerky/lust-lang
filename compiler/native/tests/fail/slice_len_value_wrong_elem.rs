// expect: L0612
// rustc E0631: `<[u16]>::len` is `fn(&[u16]) -> usize`; the bound wants `&[u32]`.
fn read<R>(v: &[u32], f: impl FnOnce(&[u32]) -> R) -> R {
    f(v)
}
fn main() {
    let v = vec![1u32];
    println!("{}", read(&v, <[u16]>::len));
}
