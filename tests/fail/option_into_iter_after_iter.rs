// expect: L0311
// rustc E0505: `into_iter` moves the Option its `iter` still borrows.
fn main() {
    let o = Some(String::from("x"));
    let it = o.iter();
    let owned: Vec<String> = o.into_iter().collect();
    println!("{:?} {:?}", it.count(), owned);
}
