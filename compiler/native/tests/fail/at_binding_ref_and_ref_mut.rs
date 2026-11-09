// expect: L0303
// rustc: cannot borrow value as mutable because it is also borrowed as immutable.
fn main() {
    let mut v = (1u8, 2u8);
    match v {
        ref a @ (_, ref mut b) => {}
    }
    println!("{:?}", v);
}
