// expect: L0311
// rustc E0505: `fold`'s result borrows `v`, which moves first.

fn main() {
    let v = vec![String::from("aa"), String::from("b"), String::from("ccc")];
    let longest = v.iter().fold("", |acc, s| if s.len() > acc.len() { s.as_str() } else { acc });
    drop(v);
    println!("{}", longest);
}
