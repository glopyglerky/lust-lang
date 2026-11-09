// expect: L0311
// rustc E0505: the closure borrows `s` until its last call; `s` moves
// first.
fn main() {
    let s = String::from("s");
    let get = || &s;
    drop(s);
    println!("{}", get());
}
