// expect: L0312
// rustc E0510: the guard writes a tested field of the scrutinee.
fn main() {
    let mut t = (1, 2);
    match t { (1, _) if { t.0 = 5; true } => println!("x"), _ => {} }
    println!("{:?}", t);
}
