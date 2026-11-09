// expect: L0312
// rustc E0506: the match holds a borrow of the scrutinee during the guard.
fn f(mut x: Option<u8>) -> u8 {
    match x {
        Some(y) if { x = None; true } => y,
        _ => 1,
    }
}
fn main() {
    println!("{}", f(Some(4)));
}
