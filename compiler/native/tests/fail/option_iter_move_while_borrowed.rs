// expect: L0311
// rustc E0505: the Option moves while its Iter borrows it.
fn main() {
    let o = Some(String::from("x"));
    let it = o.iter();
    let taken = o;
    println!("{:?} {:?}", it.count(), taken);
}
