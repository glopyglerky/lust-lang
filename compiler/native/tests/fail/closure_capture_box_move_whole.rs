// expect: L0301
// Moving a field out of a box in a closure moves the whole box.
struct S { v: Vec<u32>, y: u32 }
fn main() { let b = Box::new(S { v: vec![1], y: 2 }); let f = || { let v = b.v; v.len() }; println!("{}", f()); println!("{}", b.y); }
