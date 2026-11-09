// expect: L0110
// A shared reference does not coerce to a mutable raw pointer.
fn main() {
    let x = 4;
    let p: *mut i32 = &x;
    println!("{}", p.is_null());
}
