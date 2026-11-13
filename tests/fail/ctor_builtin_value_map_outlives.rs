// expect: L0311
// rustc E0505: `r.map(Some)` keeps the borrow inside `r`.
fn main() {
    let data = vec![1u8, 2];
    let r: Result<&u8, ()> = Ok(&data[0]);
    let o = r.map(Some);
    drop(data);
    println!("{:?}", o);
}
