// `bool::then_some` and `bool::then` (W-BENCH4 rank 23): the eager
// value drops on `false`, the closure runs only on `true`, and a
// moved capture drops with the closure either way.
use std::cell::Cell;
use std::rc::Rc;
struct D(Rc<Cell<u32>>, u32);
impl Drop for D {
    fn drop(&mut self) {
        self.0.set(self.0.get() + self.1);
    }
}
fn main() {
    println!("{:?} {:?}", (1 > 0).then_some(3), (1 > 2).then_some(3));
    println!("{:?} {:?}", true.then(|| 4), false.then(|| 4));
    let drops = Rc::new(Cell::new(0));
    let v = false.then_some(D(drops.clone(), 5));
    println!("{} {}", v.is_none(), drops.get());
    let w = true.then_some(D(drops.clone(), 7));
    println!("{} {}", w.is_some(), drops.get());
    drop(w);
    println!("{}", drops.get());
    let cap = String::from("cap");
    let x = true.then(move || cap.len());
    println!("{x:?}");
    let runs = Cell::new(0);
    let _y = false.then(|| runs.set(runs.get() + 1));
    println!("{}", runs.get());
    let s = "hi".len() == 2;
    println!("{:?}", s.then_some(String::from("yes")));
}
