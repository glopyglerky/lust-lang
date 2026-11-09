// expect: L0312
// rustc E0510: a guard cannot assign the scrutinee, even with no binding.
fn main() {
    let mut o = Some(1);
    match o { Some(_) if { o = None; false } => {}, _ => println!("x") }
    println!("{:?}", o);
}
