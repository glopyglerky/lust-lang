use std::marker::PhantomData;
struct K<T>(u32, PhantomData<T>);
impl<T: Copy> K<T> {
    const fn new(v: u32) -> K<T> {
        K(v, PhantomData)
    }
}
static A: K<String> = K::new(1);
fn main() {
    println!("{}", A.0);
}
