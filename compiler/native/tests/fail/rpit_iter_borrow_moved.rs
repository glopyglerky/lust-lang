// expect: L0311
// rustc E0505: an impl Iterator + '_ borrows self (W-CHECK-G3).
struct C { v: Vec<u32> }
impl C { fn items(&self) -> impl Iterator<Item = &u32> + '_ { self.v.iter() } }
fn main() {
    let c = C { v: vec![1, 2] };
    let it = c.items();
    drop(c);
    for x in it { println!("{}", x); }
}
