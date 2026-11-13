use std::rc::Rc;
use std::cell::RefCell;
struct P { a: Vec<u8>, b: u32 }
fn main() {
    let r = Rc::new(P { a: vec![1], b: 2 });
    let f = || r.a.len() + r.b as usize;
    println!("{} {}", f(), Rc::strong_count(&r));
    let g = move || r.b;
    println!("{}", g());
    let c = RefCell::new(P { a: vec![], b: 0 });
    let mut bm = c.borrow_mut();
    let mut h = || { bm.b += 5; bm.a.push(1); };
    h(); h();
    println!("{} {:?}", bm.b, bm.a);
}
