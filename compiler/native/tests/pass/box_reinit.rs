// After *b moves out, assigning through the box refills it;
// the allocation is freed once at the end.
struct G { n: i32, s: String }
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.n); } }
fn main() {
    let mut b = Box::new(G { n: 1, s: "a".to_string() });
    let g = *b;
    println!("out {}", g.n);
    *b = G { n: 2, s: "b".to_string() };
    println!("back {} {}", b.n, b.s);
}
