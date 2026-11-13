// Assigning through a Box drops the old pointee first.
struct G { n: i32 }
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.n); } }
fn main() {
    let mut b = Box::new(G { n: 1 });
    *b = G { n: 2 };
    println!("now {}", b.n);
    b = Box::new(G { n: 3 });
    println!("now {}", b.n);
}
