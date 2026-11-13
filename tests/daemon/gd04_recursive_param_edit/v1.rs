struct S<T>(Option<Box<S<T>>>, T);
fn inner(s: &S<u8>) -> u8 {
    match &s.0 {
        Some(b) => b.1,
        None => 0,
    }
}
fn main() {
    let s = S(Some(Box::new(S(None, 1u8))), 2u8);
    println!("{}", s.1 + inner(&s));
}
