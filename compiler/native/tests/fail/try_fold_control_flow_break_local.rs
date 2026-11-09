// expect: L0306
// rustc E0597: the `Break` payload borrows an item of `v`, which is dropped
// before `r` is used.
use std::ops::ControlFlow;
fn main() {
    let r;
    {
        let v = vec![String::from("a"), String::from("bb")];
        r = v.iter().try_fold((), |(), s| if s.len() > 1 { ControlFlow::Break(s) } else { ControlFlow::Continue(()) });
    }
    println!("{:?}", r);
}
