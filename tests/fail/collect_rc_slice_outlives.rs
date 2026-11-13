// expect: L0306
// rustc E0597: the collected Rc<[&str]> borrows `s`, which is dropped while the Rc is still used.
use std::rc::Rc;
fn main() {
    let r: Rc<[&str]>;
    {
        let s = String::from("a b");
        r = s.split(' ').collect();
    }
    println!("{r:?}");
}
