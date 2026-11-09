// expect: L0110
// rustc E0596: an `Rc` gives only shared access, so its FnMut closure cannot be called.
use std::rc::Rc;

fn main() {
    let mut n = 0u32;
    let r = Rc::new(|| {
        n += 1;
        n
    });
    println!("{}", r());
}
