// feature: std.runtime
// form: PhantomData field that marks a type param, as in ts-rust stdio markers
use std::marker::PhantomData;
struct Keep<T> { n: i32, marker: PhantomData<T> }
fn main() {
    let k = Keep::<String> { n: 3, marker: PhantomData };
    println!("{}", k.n);
}
