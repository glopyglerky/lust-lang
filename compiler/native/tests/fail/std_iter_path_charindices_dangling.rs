// expect: L0307
// rustc E0515: the iterator borrows a local that the fn drops.
fn dangle() -> std::str::CharIndices<'static> {
    let s = String::from("ab");
    s.char_indices()
}

fn main() {
    println!("{}", dangle().count());
}
