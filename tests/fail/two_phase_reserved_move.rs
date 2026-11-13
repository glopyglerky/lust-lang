// expect: L0311
// rustc E0505: a two-phase receiver is a shared borrow while the
// arguments run, so moving a field of the receiver out still conflicts.
struct S { name: String }
impl S {
    fn set(&mut self, n: String) { self.name = n; }
}
fn main() {
    let mut s = S { name: String::new() };
    s.set(s.name);
    println!("{}", s.name);
}
