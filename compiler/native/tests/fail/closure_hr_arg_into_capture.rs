// expect: L0733
// rustc E0521: the closure's `&dyn Show` argument is higher-ranked, so it cannot
// go into the captured `Vec<&dyn Show>` (a known capture-outlives-env bound must
// not prove it).
trait Show {
    fn show(&self) -> u8;
}
struct N(u8);
impl Show for N {
    fn show(&self) -> u8 {
        self.0
    }
}
fn main() {
    let mut keep: Vec<&dyn Show> = Vec::new();
    let mut f: Box<dyn FnMut(&dyn Show)> = Box::new(|s| keep.push(s));
    let n = N(3);
    f(&n);
    println!("ok");
}
