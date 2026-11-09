// expect: L0311
// rustc E0505: a `move` closure returns the reference it captured by
// value; the result borrows `s`.
fn main() {
    let s = String::from("s");
    let r = &s;
    let c = move || r;
    let x = c();
    drop(s);
    println!("{}", x);
}
