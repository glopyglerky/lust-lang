// A generic fn named by a qualified path, as a value: the written
// Self type fixes the impl's params (was L0614).
struct W<T>(T);
impl<T> W<T> {
    fn get() -> u8 {
        4
    }
}
fn main() {
    let f = <W<u16>>::get;
    println!("{}", f());
}
