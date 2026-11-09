// expect: L0110
// rustc E0107/E0599: without `std` there is no default hasher and no `new`.
fn main() {
    let m: indexmap::IndexMap<i32, i32> = indexmap::IndexMap::new();
    println!("{}", m.len());
}
