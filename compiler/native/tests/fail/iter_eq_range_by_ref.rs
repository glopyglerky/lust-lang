// expect: L0612
// rustc E0277: `(0..3).eq(..)` is Iterator::eq (by value), and `&Range` is not an iterator.
fn main() {
    let r = (0..3).eq(&(0..3));
    println!("{}", r);
}
