// `fold`'s accumulator and items are free: the result borrows `v`.
fn main() {
    let v = vec![String::from("aa"), String::from("b"), String::from("ccc")];
    let longest = v.iter().fold("", |acc, s| if s.len() > acc.len() { s.as_str() } else { acc });
    println!("{}", longest);
}
