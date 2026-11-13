const fn n() -> usize { "bad" }
const N: usize = n();
const M: usize = N + 1;
struct S {
    a: [u8; M],
}
fn main() {
    let s = S { a: [7; M] };
    let b = [0u32; N];
    println!("{} {} {} {}", s.a.len(), b.len(), std::mem::size_of::<S>(), s.a[M - 1]);
}
