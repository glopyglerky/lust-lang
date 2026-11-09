// expect: L0312
// rustc E0506: `b` is dropped at the end of `main`, and the drop of a
// `Box<dyn Fn + '_>` may use the borrow it holds.
fn main() {
    let mut x = 0;
    let b: Box<dyn Fn() -> i32 + '_> = Box::new(|| x);
    println!("{}", b());
    x += 1;
    println!("{}", x);
}
