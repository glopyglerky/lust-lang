// panic: capacity overflow
// reserve_exact with a len + extra sum past isize::MAX aborts with a
// capacity overflow, as in rustc (it used to wrap and segfault).
fn main() {
    let mut s = String::from("a");
    s.reserve_exact(usize::MAX);
    println!("{}", s);
}
