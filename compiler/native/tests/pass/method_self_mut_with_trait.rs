struct C {
    n: u32,
}
impl C {
    fn inc(&mut self) {
        self.n += 1;
    }
    fn twice(&mut self) {
        self.inc();
        self.inc();
    }
}
trait Tr {
    fn t(&self) -> u32 {
        0
    }
}
fn main() {
    let mut c = C { n: 0 };
    c.twice();
    println!("{}", c.n);
}
