// panic: capacity overflow
// reserve with a len + extra sum that wraps usize aborts with a
// capacity overflow, as in rustc (it used to wrap and keep going).
fn main() {
    let mut s = String::from("abc");
    s.reserve(usize::MAX - 2);
    println!("{}", s);
}
