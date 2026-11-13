// expect: L0308
trait Marker {}
struct C {
    n: u32,
}
impl C {
    fn inc(&mut self) {
        self.n += 1;
    }
}
fn main() {
    let c = C { n: 0 };
    let r = &c;
    r.inc();
}
