// feature: supported.std_core
// form: Vec, String and Option basic methods
fn main() {
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    let mut s = String::from("ab");
    s.push('c');
    println!("{} {} {}", v.len(), s.len(), Some(2).unwrap_or(0));
}
