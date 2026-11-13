struct C {
    n: u32,
}
impl C {
    fn inc(&mut self) {
        self.n += 1;
    }
}
trait Tr {
    fn t(&self) -> u32 {
        0
    }
}
fn bump(c: &mut C) {
    c.inc();
    c.inc();
}
fn main() {
    let mut c = C { n: 7 };
    bump(&mut c);
    println!("{}", c.n);
}
