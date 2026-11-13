// expect: L0110
// `Vec<T>: Extend<&T>` needs `T: Copy`: `&String` items do not fill a `Vec<String>`.
fn main() {
    let v = vec!["a".to_string(), "bb".to_string()];
    let (a, b): (Vec<String>, Vec<String>) = v.iter().partition(|s| s.len() == 1);
    println!("{:?} {:?}", a, b);
}
