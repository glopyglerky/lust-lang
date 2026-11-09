// expect: L0308
// rustc E0596: an arm after a catch-all is still borrow-checked (false edge).
fn main() {
    let v = 5;
    match v {
        _ => println!("a"),
        ref mut x => *x += 1,
    }
    println!("{}", v);
}
