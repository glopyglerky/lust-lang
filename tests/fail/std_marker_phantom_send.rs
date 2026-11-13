// expect: L0612
// rustc E0277: `PhantomData<*const ()>` makes the struct `!Send`.
use std::marker::PhantomData;
struct NotSend {
    _m: PhantomData<*const ()>,
}
fn need<T: Send>(_: T) {}
fn main() {
    need(NotSend { _m: PhantomData });
}
