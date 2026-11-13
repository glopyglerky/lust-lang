// expect: L0311
// rustc E0505: the mapped items borrow `v`, which moves while they live.

fn main() {
    let v = vec![String::from("a"), String::from("b")];
    let r: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
    drop(v);
    println!("{:?}", r);
}
