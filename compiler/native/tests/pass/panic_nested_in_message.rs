// A panic while a panic message is formatted (a Debug impl that panics
// inside assert_eq!) is a panic in the panic hook: Rust prints the inner
// panic's static message and "thread panicked while processing panic".
// panic: thread panicked while processing panic. aborting.
#[derive(PartialEq)]
struct S(i32);
impl std::fmt::Debug for S {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        panic!("debug panic")
    }
}
fn main() {
    println!("start");
    assert_eq!(S(1), S(bb(2)));
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
