// expect: L0202
// rustc E0106: an elided lifetime in a returned `impl Trait`'s bounds
// follows the output elision rules, and two reference inputs leave
// none to take.
fn both(a: &[u8], b: &[u8]) -> impl Iterator<Item = &u8> {
    a.iter().chain(b.iter())
}
fn main() {
    println!("{}", both(&[1], &[2]).count());
}
