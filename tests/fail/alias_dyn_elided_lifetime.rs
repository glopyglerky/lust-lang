// expect: L0202
// rustc E0106: a type alias cannot elide a dyn's lifetime bound.
type A = Box<dyn Fn() -> u8 + '_>;
fn main() {
    let x: A = Box::new(|| 1);
    println!("{}", x());
}
