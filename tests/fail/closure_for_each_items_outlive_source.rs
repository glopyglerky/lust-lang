// expect: L0311
// rustc E0505: the items stored in `out` borrow `v`, which moves while
// `out` is used.
fn main() {
    let v = vec![1, 2, 3];
    let mut out: Vec<&i32> = Vec::new();
    v.iter().for_each(|x| out.push(x));
    drop(v);
    println!("{}", out.len());
}
