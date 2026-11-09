// ARCH 2.8: a dyn call that returns a large struct (out-pointer ABI).
#[derive(Debug, Clone, Copy)]
struct Big { a: u64, b: u64, c: u64, d: u64, e: u64 }
trait Make { fn make(&self, k: u64) -> Big; fn small(&self) -> u8 { 3 } }
struct M(u64);
impl Make for M { fn make(&self, k: u64) -> Big { Big { a: self.0, b: k, c: self.0 + k, d: 4, e: 5 } } }
fn main() {
    let m = M(10);
    let d: &dyn Make = &m;
    let b = d.make(2);
    println!("{:?} {}", b, d.small());
}
