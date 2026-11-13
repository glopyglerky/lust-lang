// expect: L0110
// rustc E0308: a user type named Ordering is not std::cmp::Ordering.
#[derive(Debug)]
enum Ordering { Less, More }
fn f() -> std::cmp::Ordering { Ordering::Less }
fn main() { println!("{:?} {:?}", f(), Ordering::More); }
