// A by-value use of one tuple field captures only that field: the
// other fields stay usable in the parent.
fn consume(s: String) -> usize {
    s.len()
}
fn main() {
    let t = (String::from("first"), String::from("second"), 3u8);
    let c = move || consume(t.0);
    println!("t.1 still here: {}", t.1);
    println!("t.2 still here: {}", t.2);
    println!("call {}", c());
    let mut pair = (vec![1u8], vec![2u8, 3]);
    let mut grow = || pair.1.push(9);
    grow();
    pair.0.push(7);
    println!("{:?}", pair);
}
