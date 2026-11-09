// std's Option::expect formats its text as a `{}` argument, so in a
// nested panic (inside a Display impl while a panic message formats)
// the report shows an empty message.
// panic: thread panicked while processing panic. aborting.
struct S;
impl std::fmt::Display for S {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let o: Option<u8> = bb(None);
        o.expect("exp");
        Ok(())
    }
}
fn main() {
    panic!("outer {}", S);
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
