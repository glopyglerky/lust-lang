// expect: L0309
// rustc E0594: assignment through a Cow (no DerefMut).
use std::borrow::Cow;

fn main() {
    let mut c: Cow<i32> = Cow::Owned(5);
    *c = 6;
    println!("{c}");
}
