// expect: L0303
// rustc E0502: the closure borrows `*d` while `d.inc()` needs it mutably.
trait Counter {
    fn inc(&mut self) -> u32;
    fn get(&self) -> u32;
}
impl Counter for u32 {
    fn inc(&mut self) -> u32 {
        *self += 1;
        *self
    }
    fn get(&self) -> u32 {
        *self
    }
}
fn main() {
    let mut n = 7u32;
    let d: &mut dyn Counter = &mut n;
    let f = || d.get();
    d.inc();
    println!("{}", f());
}
