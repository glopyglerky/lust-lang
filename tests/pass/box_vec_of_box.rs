// Boxes as Vec elements: indexing through two derefs,
// remove, and drop of the removed box.
struct G { n: i32 }
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.n); } }
fn main() {
    let mut v: Vec<Box<G>> = Vec::new();
    v.push(Box::new(G { n: 1 }));
    v.push(Box::new(G { n: 2 }));
    println!("{} {}", v[0].n, v.len());
    let first = v.remove(0);
    println!("removed {}", first.n);
    drop(first);
    println!("rest {}", v[0].n);
}
