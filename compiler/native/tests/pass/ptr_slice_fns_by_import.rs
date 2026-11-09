// std::ptr and std::slice free fns by glob, explicit import, alias,
// block glob, and module re-export (they worked only by full path).
use std::ptr::*;
use std::slice::from_mut as one_mut;
mod r {
    pub use std::ptr::null_mut;
}
fn main() {
    let a: *const u8 = null();
    let b = r::null_mut::<i32>();
    let x = 5;
    let y = 5;
    println!("{} {} {} {}", a.is_null(), b.is_null(), eq(&x, &x), eq(&x, &y));
    let arr = [1u8, 2];
    println!("{}", addr_eq(&arr[0], &arr[0] as *const u8));
    let mut z = 3;
    one_mut(&mut z)[0] = 4;
    {
        use core::slice::*;
        println!("{:?} {}", from_ref(&z), from_ref::<u8>(&arr[1]).len());
    }
    let p = from_ref(&z);
    println!("{}", p.is_null());
}
