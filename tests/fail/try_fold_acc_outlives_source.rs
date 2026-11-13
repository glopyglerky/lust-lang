// expect: L0311
// rustc E0505: the accumulator holds a borrow of an item, so `v` cannot move while `r` lives.
fn main() {
    let v = vec![String::from("ab"), String::from("c")];
    let r = v.iter().try_fold("", |a, s| Some(if s.len() > a.len() { s.as_str() } else { a }));
    let w = v;
    println!("{:?} {}", r, w.len());
}
