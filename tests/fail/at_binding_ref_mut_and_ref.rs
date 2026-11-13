// expect: L0303
// rustc: cannot borrow value as immutable because it is also borrowed as mutable.
fn main() { let mut x = (1, 2); match x { ref mut a @ (_, ref b) => {}, } }
