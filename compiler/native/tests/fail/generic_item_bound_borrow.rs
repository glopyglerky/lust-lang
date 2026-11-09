// expect: L0311
// rustc E0505: `T` is `&String` through `Item = T`, so the result borrows
// `v` (this was a false accept and a use after free).
fn mk<I: Iterator<Item = T>, T>(i: I) -> Vec<T> {
    let mut o = Vec::new();
    for x in i {
        o.push(x);
    }
    o
}
fn main() {
    let v = vec![String::new()];
    let s = mk(v.iter());
    drop(v);
    println!("{}", s.len());
}
