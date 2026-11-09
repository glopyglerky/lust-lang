// str::make_ascii_uppercase and make_ascii_lowercase on String, str ranges, and Vec elements (R3).
fn main() {
    let mut s = String::from("Hello, Wörld!"); s.make_ascii_uppercase(); println!("{s}");
    s[..5].make_ascii_lowercase(); println!("{s}");
    let mut v = vec![String::from("x")]; v[0].make_ascii_uppercase(); println!("{v:?}");
}
