// expect: L0110
// rustc E0277: `B` does not derive or implement Hash, so `A` cannot
// derive Hash over a `B` field, even though `B` points back at `A`.
#[derive(Hash)]
struct A { b: Option<Box<B>> }
struct B { a: Option<Box<A>> }
fn main() { println!("x"); }
