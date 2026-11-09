// expect: L0312
// rustc E0506: overwriting a `Box<dyn Fn + '_>` drops the old object,
// whose unknown destructor may use the borrow of `x`.
fn main() {
    let mut x = 1;
    let y = 2;
    let mut b: Box<dyn Fn() -> i32 + '_> = Box::new(|| x);
    println!("{}", b());
    x = 5;
    b = Box::new(|| y);
    println!("{} {}", x, b());
}
