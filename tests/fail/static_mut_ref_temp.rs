// expect: L0631
// rustc E0764: a promoted mutable borrow of a temporary.
static S: &mut i32 = &mut 5;
fn main() { println!("{}", S); }
