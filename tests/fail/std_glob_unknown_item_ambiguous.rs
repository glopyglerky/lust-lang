// expect: L0507
// rustc E0659: `std::mem::*` and `std::ptr::*` both bring a `swap`, in a
// block too.
fn main() {
    use std::mem::*;
    #[allow(unused_imports)]
    use std::ptr::*;
    let mut a = 1;
    let mut b = 2;
    swap(&mut a, &mut b);
    println!("{a} {b}");
}
