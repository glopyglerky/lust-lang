// `map`'s closure argument is an item (free): returning a borrow of it
// is fine while the items' source lives.
fn main() {
    let v = vec![String::from("a")];
    let s: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
    println!("{:?}", s);
}
