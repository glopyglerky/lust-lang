// expect: L0612
// Valid Rust: PhantomData<F> is Debug for every F, but its text names
// type_name::<F>(), which lust cannot print for a closure type.
use std::marker::PhantomData;
fn show<T: std::fmt::Debug>(t: T) { println!("{:?}", t); }
fn main() {
    let f = |x: u8| x;
    let p = phantom_of(&f);
    show(p);
}
fn phantom_of<F>(_: &F) -> PhantomData<F> { PhantomData }
