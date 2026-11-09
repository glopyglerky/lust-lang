fn main() {
    let s = "héllo";
    println!("{}", &s[0..1]);
    println!("{}", &s[1..3]);
    println!("{}", &s[3..]);
    println!("{}", &s[..3]);
    println!("{}", &s[..]);
    println!("{}", &s[0..=0]);
    let owned = String::from("abcdef");
    println!("{}", &owned[2..4]);
    let v = &owned[1..5];
    println!("{} {}", v, v.len());
    let empty = &s[3..3];
    println!("[{}]", empty);
    let t: &str = "xyz";
    println!("{}", &t[1..2]);
}
