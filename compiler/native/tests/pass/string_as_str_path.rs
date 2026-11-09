// `String::as_str` as a path: called directly, as a fn value,
// and as an iterator adapter argument (tf-str).
fn main() {
    let s = String::from("hi");
    println!("{}", String::as_str(&s));
    let g = String::as_str;
    println!("{}", g(&s));
    let v = vec![String::from("a"), String::from("bb")];
    let w: Vec<&str> = v.iter().map(String::as_str).collect();
    println!("{:?}", w);
    println!("{:?}", v.first().map(String::as_str));
    println!("{}", v.last().map_or("none", String::as_str));
    let joined: Vec<String> = v.iter().map(String::as_str).map(|x| x.to_uppercase()).collect();
    println!("{:?}", joined);
}
