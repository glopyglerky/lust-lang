// expect: L0112
// rustc E0277: `?` on a `ControlFlow` in a fn that returns `Result`.
use std::ops::ControlFlow;
fn inner() -> ControlFlow<i32> { ControlFlow::Break(1) }
fn outer() -> Result<(), i32> {
    inner()?;
    Ok(())
}
fn main() { let _ = outer(); }
