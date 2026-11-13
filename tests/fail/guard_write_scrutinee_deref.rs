// expect: L0312
// rustc E0510: the guard writes the scrutinee through the reference it is read from.
fn main() {
    let mut x = 1;
    let p = &mut x;
    match *p { 1 if { *p = 2; true } => println!("y"), _ => {} }
}
