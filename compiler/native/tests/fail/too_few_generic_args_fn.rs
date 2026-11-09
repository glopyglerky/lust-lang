// expect: L0107
// rustc E0107: function takes 2 generic arguments but 1 was supplied.
fn pair<A, B>(a: A, b: B) -> (A, B) { (a, b) }
fn main() { let x = pair::<u8>(255, 1u8); println!("{:?}", x); }
