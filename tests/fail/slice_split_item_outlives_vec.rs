// expect: L0306
// rustc E0597: a split item cannot outlive its Vec.
fn main() { let p; { let v = vec![1, 0]; p = v.split(|x| *x == 0).next().unwrap(); } println!("{:?}", p); }
