// expect: L0303
// rustc: cannot borrow value as mutable more than once at a time.
fn main() { let mut x = (1, 2); match x { ref mut a @ (_, ref mut b) => {}, } }
