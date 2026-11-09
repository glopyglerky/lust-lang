// expect: L0303
// rustc E0499: the guard holds `v` mutably through `PhantomData<&'a mut _>`.
use std::marker::PhantomData;
struct Guard<'a> {
    _m: PhantomData<&'a mut Vec<i32>>,
}
fn lock<'a>(_v: &'a mut Vec<i32>) -> Guard<'a> {
    Guard { _m: PhantomData }
}
fn main() {
    let mut v = vec![1];
    let g = lock(&mut v);
    v.push(2);
    drop(g);
}
