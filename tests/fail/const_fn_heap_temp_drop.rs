// expect: L0631
// rustc E0493: the Vec is destroyed inside the const fn.
const fn h() -> usize { Vec::<i32>::new().len() }
fn main() { println!("{}", h()); }
