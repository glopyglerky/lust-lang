// expect: L0612
// rustc E0277: Cow<Cell<i32>> is not Sync, so it cannot be a static.
use std::borrow::Cow;
use std::cell::Cell;

static S: Cow<'static, Cell<i32>> = Cow::Owned(Cell::new(1));

fn main() {
    println!("{}", S.get());
}
