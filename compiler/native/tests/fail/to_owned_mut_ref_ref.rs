// expect: L0110
// rustc E0308: `&mut &String` is no `&Self` and `&mut &String` is not
// Clone, so `to_owned` steps to `&String` and takes it by value: `String`.
fn main() {
    let s = String::from("s");
    let mut r = &s;
    let m = &mut r;
    let o: &String = m.to_owned();
    println!("{}", o);
}
