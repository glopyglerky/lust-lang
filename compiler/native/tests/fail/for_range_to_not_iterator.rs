// expect: L0612
// rustc E0277: `RangeTo<{integer}>` is not an iterator.
fn main() {
    for i in ..3 {
        println!("{i}");
    }
}
