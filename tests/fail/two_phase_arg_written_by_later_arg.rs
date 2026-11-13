// expect: L0312
// rustc E0506: a two-phase reservation is still a shared borrow while the
// later arguments run.
struct C { n: u32 }
impl C {
    fn d(&mut self, cb: &mut dyn FnMut(&mut C, u32) -> u32) -> u32 { cb(self, { self.n += 1; 1 }) }
}
fn main() { let mut c = C { n: 0 }; println!("{}", c.d(&mut |c, k| { c.n += k; c.n })); }
