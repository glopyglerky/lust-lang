// expect: L0001
// rustc: visibility `pub` is not followed by an item. rustc parses cfg-removed
// code: the removed node here is no item, and in a module it is no tuple field.
fn main() {
    println!("x");
}
#[cfg(any())]
pub unsafe
