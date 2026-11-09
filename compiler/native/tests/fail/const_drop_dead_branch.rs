// expect: L0631
// rustc E0493: the drop check is static; a dead branch still counts.
const C: u8 = if false { let v: Vec<u8> = Vec::new(); 1 } else { 2 };
fn main() { println!("{}", C); }
