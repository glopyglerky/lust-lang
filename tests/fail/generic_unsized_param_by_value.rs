// expect: L0110
// rustc E0277: a `T: ?Sized` value cannot be a parameter.
fn f<T: ?Sized>(x: T) -> u8 {
    let _ = &x;
    1
}

fn main() {
    println!("x");
}
