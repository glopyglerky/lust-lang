// expect: L0107
// rustc E0107: the alias takes one generic argument; lust dropped the `4`.
struct S<T, const N: usize> {
    a: [T; N],
}
impl<T: Copy, const N: usize> S<T, N> {
    fn mk(v: T) -> Self {
        S { a: [v; N] }
    }
    fn n(&self) -> usize {
        self.a.len()
    }
}
type B<T> = S<T, 3>;
fn main() {
    let s = B::<u8, 4>::mk(1);
    println!("{}", s.n());
}
