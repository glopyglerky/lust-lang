// expect: L0612
// rustc E0277: `u32` is not an iterator.
fn bad() -> impl Iterator<Item = u32> {
    5u32
}

fn main() {}
