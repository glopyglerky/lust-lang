// `Vec::swap_remove` checks its index inside alloc.
// panic: swap_remove index (is 4) should be < len (is 3)
fn main() {
    let n = std::env::args().count();
    let mut v = vec![1, 2, 3, 4];
    println!("{}", v.swap_remove(n - 1));
    println!("{:?}", v);
    println!("{}", v.swap_remove(n + 3));
}
