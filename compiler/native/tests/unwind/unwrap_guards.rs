// unwrap on None with a RefMut guard and a Box live: both drop.
struct G(&'static str);
impl Drop for G { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let _a = G("a");
    let o: Option<G> = None;
    let b = Box::new(G("boxed"));
    let s = std::cell::RefCell::new(1);
    let _m = s.borrow_mut();
    if b.0.len() > 2 {
        let _r = s.borrow();
    }
    o.unwrap();
}
