// expect: L0112
// rustc E0277: `?` on `ControlFlow<i32>` in a fn that returns
// `ControlFlow<u32>`: the break types must be the same.
use std::ops::ControlFlow;
fn inner() -> ControlFlow<i32> { ControlFlow::Break(1) }
fn outer() -> ControlFlow<u32> {
    inner()?;
    ControlFlow::Continue(())
}
fn main() { let _ = outer(); }
