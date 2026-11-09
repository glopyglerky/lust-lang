// expect: L0311
// rustc E0505: `once(&s)` keeps the borrow of `s`, so `s` cannot move.
fn main() {
    let s = String::from("a");
    let it = std::iter::once(&s);
    drop(s);
    println!("{}", it.count());
}
