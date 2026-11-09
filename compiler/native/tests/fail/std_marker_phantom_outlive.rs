// expect: L0306
// rustc E0597: the iterator borrows `v` through `PhantomData<&'a T>`.
use std::marker::PhantomData;
struct Iter<'a, T> {
    i: usize,
    _m: PhantomData<&'a T>,
}
fn iter_of<'a, T>(_v: &'a [T]) -> Iter<'a, T> {
    Iter { i: 0, _m: PhantomData }
}
fn main() {
    let r;
    {
        let v = vec![1, 2];
        r = iter_of(&v);
    }
    println!("{}", r.i);
}
