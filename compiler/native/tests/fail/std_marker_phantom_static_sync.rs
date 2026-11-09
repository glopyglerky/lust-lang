// expect: L0612
// rustc E0277: a shared static needs a `Sync` type.
use std::marker::PhantomData;
struct NotSync {
    _m: PhantomData<*const ()>,
}
static X: NotSync = NotSync { _m: PhantomData };
fn main() {
    let _ = &X;
}
