// expect: L0303
// rustc E0502: the predicate holds `v` mutably while `v` is read.
fn main() {
    let mut v: Vec<char> = Vec::new();
    let r = "abc".replace(|c: char| { v.push(c); c == 'b' }, &format!("{}", v.len()));
    println!("{r}");
}
