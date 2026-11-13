// expect: L0612
// rustc E0277: the bound is checked in an untaken branch too.
use std::marker::PhantomData;
struct K<T>(u32, PhantomData<T>);
impl<T: Clone> K<T> {
    const fn new(v: u32) -> K<T> {
        K(v, PhantomData)
    }
}
struct N;
const X: u32 = if true {
    1
} else {
    let k: K<N> = K::new(2);
    k.0
};
fn main() {
    println!("{}", X);
}
