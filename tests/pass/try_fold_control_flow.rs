// `try_fold` with a `ControlFlow<B, C>` closure result: `C` is the
// accumulator, `Break(b)` stops early, and a break payload may borrow the
// items.
use std::ops::ControlFlow;
fn first_long<'a>(v: &'a [String]) -> Option<&'a String> {
    match v.iter().try_fold((), |(), s| if s.len() > 1 { ControlFlow::Break(s) } else { ControlFlow::Continue(()) }) {
        ControlFlow::Break(s) => Some(s),
        ControlFlow::Continue(()) => None,
    }
}
fn main() {
    let v = vec![1u8, 2, 3];
    let r = v.iter().try_fold(0u32, |a, &x| if x > 2 { ControlFlow::Break(a) } else { ControlFlow::Continue(a + x as u32) });
    println!("{:?}", r);
    let r: ControlFlow<u8, u32> = v.iter().try_fold(0u32, |a, &x| ControlFlow::Continue(a + x as u32));
    println!("{:?} {}", r, r.is_continue());
    let w = vec![String::from("a"), String::from("bb")];
    println!("{:?}", first_long(&w));
    let mut acc = Vec::new();
    let _ = v.iter().try_fold(&mut acc, |a, x| {
        a.push(*x);
        ControlFlow::<(), _>::Continue(a)
    });
    println!("{:?}", acc);
}
