// The inner panic's message has arguments (an index out of bounds), so
// the nested-panic report shows an empty message.
// panic: thread panicked while processing panic. aborting.
struct S(Vec<u8>);
impl std::fmt::Debug for S {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0[bb(7)])
    }
}
fn main() {
    let r: Result<u8, S> = Err(S(vec![1]));
    print!("held");
    r.unwrap();
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
