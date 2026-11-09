// The string equality forms Rust allows between String, str, and references to them.
fn main() {
    let mut s = String::from("a");
    let t = "a";
    println!("{}", s == "a");
    println!("{}", &s == "a");
    println!("{}", &mut s == "a");
    println!("{}", &s == t);
    println!("{}", t == &s);
    println!("{}", &t == &s);
    println!("{}", &s == &t);
    println!("{}", *t == s);
    println!("{}", s != "b");
    println!("{}", assert_eq_helper(&s));
}

fn assert_eq_helper(s: &String) -> bool {
    s == "a"
}
