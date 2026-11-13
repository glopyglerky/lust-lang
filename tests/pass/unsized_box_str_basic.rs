fn main() {
    let b: Box<str> = Box::from("hey");
    let c = b.clone();
    let s: String = c.into();
    println!("{b} {s} {}", b.len());
    let d: Box<str> = String::from("abc").into_boxed_str();
    let e = d.clone().into_string();
    println!("{d:?} {e}");
    let f: Box<str> = "q".into();
    println!("{}", f.starts_with('q'));
}
