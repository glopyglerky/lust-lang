// A fn that starts on the line where a patch edit ends moves in its
// line, and its text hash reads that line from its start. v2 and v3
// edit S's method body (the body patch), v3 also changes the width;
// v4 edits get's header (the item patch). Each time main hashes again,
// so the column `here()` reports, and a panic location, are current
// (hunt8 12:53: Rc<dyn T> in main, warm run kept main's old column).
use std::panic::Location;
use std::rc::Rc;
#[track_caller]
fn here() -> u32 { Location::caller().column() }
trait T { fn v(&self) -> i32; }
struct S;
impl T for S { fn v(&self) -> i32 { 100 } } fn get(r: &dyn T) -> i32 { r.v() } fn main() { let r: Rc<dyn T> = Rc::new(S); println!("{} {}", get(&*r), here()); }
