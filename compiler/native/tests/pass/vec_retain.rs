// `Vec::retain` and `Vec::retain_mut` (W-BENCH4 rank 15): the
// predicate runs in order, a rejected element drops before the next
// call, and the kept elements keep their order.
use std::cell::Cell;
use std::rc::Rc;
struct D(u8, Rc<Cell<u32>>);
impl Drop for D {
    fn drop(&mut self) {
        self.1.set(self.1.get() * 10 + self.0 as u32);
    }
}
fn main() {
    let mut v = vec![1u8, 2, 3, 4, 5, 6];
    v.retain(|x| x % 2 == 0);
    println!("{v:?}");
    let keep = 3u8;
    let mut s = vec!["a".to_string(), "abcd".to_string(), "xy".to_string(), "longer".to_string()];
    s.retain(|x| x.len() as u8 > keep);
    println!("{s:?}");
    let seen = Rc::new(Cell::new(0u32));
    let mut d = vec![D(1, seen.clone()), D(2, seen.clone()), D(3, seen.clone()), D(4, seen.clone())];
    d.retain(|x| x.0 % 2 == 1);
    println!("{} {}", d.len(), seen.get());
    println!("{} {}", d[0].0, d[1].0);
    drop(d);
    println!("{}", seen.get());
    let mut m = vec![1u8, 2, 3];
    m.retain_mut(|x| {
        *x *= 10;
        *x != 20
    });
    println!("{m:?}");
    let mut e: Vec<u8> = vec![];
    e.retain(|_| false);
    println!("{}", e.len());
    let mut all = vec![7u8, 8];
    all.retain(|_| true);
    println!("{all:?}");
}
