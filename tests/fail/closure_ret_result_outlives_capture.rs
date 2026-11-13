// expect: L0311
// rustc E0505: the closure's result borrows `s` (its capture), and is
// used after `s` moves.
fn main() {
    let s = String::from("s");
    let get = || &s;
    let r = get();
    drop(s);
    println!("{}", r);
}
