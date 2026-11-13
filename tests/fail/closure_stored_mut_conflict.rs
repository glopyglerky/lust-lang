// expect: L0303
// rustc E0499: the stored FnMut closure holds `v` mutably.
struct Holder<F: FnMut() -> usize> {
    f: F,
}

fn main() {
    let mut v = vec![1u8];
    let mut h = Holder { f: || {
        v.push(2);
        v.len()
    } };
    v.push(3);
    println!("{}", (h.f)());
}
