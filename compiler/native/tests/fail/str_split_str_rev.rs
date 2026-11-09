// expect: L0612
// rustc E0277: a `&str` needle's searcher is not double-ended.
fn main() {
    println!("{:?}", "a b".split("b").rev().count());
}
