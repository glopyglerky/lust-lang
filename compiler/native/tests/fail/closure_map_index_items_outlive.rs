// expect: L0311
// rustc E0505: the mapped items borrow `names` through the closure;
// `names` moves while they are live.
fn main() {
    let names = vec![String::from("a"), String::from("b")];
    let idx = vec![1usize, 0];
    let picked: Vec<&String> = idx.iter().map(|&i| &names[i]).collect();
    drop(names);
    println!("{:?}", picked);
}
