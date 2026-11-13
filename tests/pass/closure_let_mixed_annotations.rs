// A `let` closure with no expected signature: an annotated parameter
// with an elided lifetime is higher-ranked, an unannotated one is free,
// as rustc infers them (rustc accepts each).
fn main() {
    let s = String::from("abc");
    let mut out: Vec<&String> = Vec::new();
    let mut f = |x: &String, y| {
        out.push(y);
        x.len()
    };
    let t = String::from("de");
    let n = f(&t, &s);
    let m = f(&s, &s);
    println!("{} {} {}", n, m, out.len());
    let count = |v: Vec<_>| v.len();
    println!("{}", count(vec![&s, &t]));
}
