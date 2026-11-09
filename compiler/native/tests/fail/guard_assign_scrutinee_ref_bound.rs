// expect: L0312
// rustc E0506: x borrows o, and the guard overwrites o.
fn main() {
    let mut o = Some(String::from("hi"));
    match o {
        Some(ref x) if { o = None; true } => println!("{}", x),
        _ => {}
    }
}
