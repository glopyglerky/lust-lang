// Drop types are fine as a const item's final value, moved out of a
// const fn whole, and behind a promoted shared borrow.
struct D(u8);
impl Drop for D { fn drop(&mut self) {} }
const fn id(d: D) -> D { d }
const C: D = id(D(3));
const S: &D = &D(1);
const V: Vec<u8> = Vec::new();
fn main() { let _ = S; println!("{} {} {}", C.0, S.0, V.len()); }
