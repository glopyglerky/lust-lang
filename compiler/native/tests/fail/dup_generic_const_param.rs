// expect: L0106
struct S<const N: usize, const N: usize> {
    x: [i64; N],
}
fn main() {}
