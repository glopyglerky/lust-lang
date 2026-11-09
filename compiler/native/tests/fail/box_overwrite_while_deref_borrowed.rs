// expect: L0312
// rustc E0506: assigning `b` drops the old Box that `r` points into.
fn main() {
    let mut b = Box::new(5);
    let r = &*b;
    b = Box::new(6);
    println!("{} {}", r, b);
}
