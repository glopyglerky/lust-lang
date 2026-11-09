// expect: L0631
// rustc E0493: the Vec local is destroyed during const evaluation.
const M: usize = { let v: Vec<u8> = Vec::new(); v.len() };
fn main() { println!("{}", M); }
