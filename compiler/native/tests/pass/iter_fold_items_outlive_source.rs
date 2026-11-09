// `fold`'s result is its initial value or what the closure returns:
// here the items' inner `&'a str`, not the borrow of the local `v`
// (rustc accepts).
fn longest<'a>(src: &[&'a str]) -> &'a str {
    let v: Vec<&'a str> = src.to_vec();
    v.iter().fold("", |a, x| if x.len() > a.len() { *x } else { a })
}
fn main() {
    let s = String::from("abc");
    let t = String::from("de");
    println!("{}", longest(&[s.as_str(), t.as_str()]));
}
