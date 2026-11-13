// panic: capacity overflow
// with_capacity above isize::MAX aborts with a capacity overflow,
// as in rustc (it used to try the allocation).
fn main() {
    let s = String::with_capacity(usize::MAX);
    println!("{}", s.capacity());
}
