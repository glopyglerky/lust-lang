// expect: L0112
// rustc E0369: a derived PartialEq on `G<T>` bounds `T: PartialEq`
// even when the only field naming T is a PhantomData, so the tuple
// struct `Out(G<Leaf>)` cannot derive PartialEq.
use std::marker::PhantomData;
#[derive(PartialEq)]
struct G<T>(PhantomData<T>, u8);
struct Leaf;
#[derive(PartialEq)]
struct Out(G<Leaf>, u8);
fn main() {
    let o = Out(G(PhantomData, 1), 2);
    println!("{}", o.1 + (o.0).1);
}
