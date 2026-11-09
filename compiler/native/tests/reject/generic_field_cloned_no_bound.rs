// expect: L0612
// Valid Rust: a struct may hold a `Cloned<slice::Iter<T>>` without `T: Clone` as long as it
// never iterates it. lust asks a written adapter type to be an iterator where it is written,
// since its builtin uses (`next`, `for`) do not prove the impl's where clauses.
struct S<'a, T> {
    it: std::iter::Cloned<std::slice::Iter<'a, T>>,
}
fn main() {
    let v = [1u8];
    let s = S { it: v.iter().cloned() };
    println!("{}", std::mem::size_of_val(&s) > 0);
}
