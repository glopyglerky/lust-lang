// expect: L0303
// rustc E0502: the iterator borrows self until it is dropped (W-CHECK-G3).
struct C { v: Vec<u32> }
impl C { fn items(&self) -> impl Iterator<Item = &u32> + '_ { self.v.iter() } }
fn main() { let mut c = C { v: vec![1, 2] }; let it = c.items(); c.v.push(3); let _ = it; }
