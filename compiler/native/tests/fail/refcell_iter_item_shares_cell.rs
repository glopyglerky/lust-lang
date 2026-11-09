// expect: L0306
// An iterator item &RefCell is the vec's own cell.
use std::cell::RefCell;
fn main() {
    let x = 5;
    let v = vec![RefCell::new(&x)];
    {
        let y = 6;
        for c in v.iter() {
            c.replace(&y);
        }
    }
    println!("{}", v[0].borrow());
}
