// Mapped items borrow `names` through the closure: `&names[i]`.
fn main() {
    let names = vec![String::from("a"), String::from("b")];
    let idx = vec![1usize, 0];
    let picked: Vec<&String> = idx.iter().map(|&i| &names[i]).collect();
    println!("{:?}", picked);
}
