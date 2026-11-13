// The pointee drops exactly when the last strong Rc drops.
use std::rc::Rc;
struct G { n: i32 }
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.n); } }
fn main() {
    let a = Rc::new(G { n: 1 });
    {
        let b = a.clone();
        let c = b.clone();
        println!("count {}", Rc::strong_count(&a));
        drop(c);
        println!("count {}", Rc::strong_count(&a));
    }
    println!("count {}", Rc::strong_count(&a));
}
