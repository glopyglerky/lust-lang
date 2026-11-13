// `Ord::clamp` on a non-primitive type asserts inside core's cmp.rs,
// not at the caller.
// panic: assertion failed: min <= max
fn main() {
    let n = std::env::args().count();
    println!("{:?}", "m".clamp("a", "k"));
    println!("{:?}", ("a", n).clamp(("a", 3), ("b", 0)));
    println!("{:?}", String::from("q").clamp(String::from("z"), String::from("b")));
}
