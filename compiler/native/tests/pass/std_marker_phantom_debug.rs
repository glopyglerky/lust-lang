// `PhantomData`'s Debug prints `PhantomData<{type_name::<T>()}>`.
use std::marker::PhantomData;
fn main() {
    let p: PhantomData<u8> = PhantomData;
    println!("{:?}", p);
}
