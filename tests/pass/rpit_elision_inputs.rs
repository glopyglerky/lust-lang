// Elided lifetimes in a returned `impl Trait`'s bounds that rustc
// accepts: one reference input, `&self`, a named lifetime, and `Fn`
// sugar, which is its own elision scope.
struct S {
    v: Vec<u8>,
}
impl S {
    fn first(&self, _skip: &[u8]) -> impl Iterator<Item = &u8> + '_ {
        self.v.iter()
    }
}
fn one(a: &[u8], n: usize) -> impl Iterator<Item = &u8> {
    a.iter().take(n)
}
fn named<'a>(a: &'a [u8], _b: &[u8]) -> impl Iterator<Item = &'a u8> {
    a.iter()
}
fn owned(a: &[u8], b: &[u8]) -> impl Iterator<Item = u8> {
    a.to_vec().into_iter().chain(b.to_vec())
}
fn main() {
    let s = S { v: vec![1, 2, 3] };
    let a = [4u8, 5];
    println!("{}", s.first(&a).count());
    println!("{}", one(&a, 1).count());
    println!("{}", named(&a, &a).count());
    println!("{}", owned(&a, &a).count());
}
