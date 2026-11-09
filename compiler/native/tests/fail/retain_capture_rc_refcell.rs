// expect: L0306
// rustc E0597: the predicate stores its capture into the elements through `Rc<RefCell<..>>`.
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let mut v: Vec<Rc<RefCell<Vec<&u32>>>> = vec![Rc::new(RefCell::new(Vec::new()))];
    let w = v.clone();
    {
        let k = 3u32;
        let c = &k;
        v.retain(|e| {
            e.borrow_mut().push(c);
            true
        });
    }
    println!("{}", w[0].borrow().len());
}
