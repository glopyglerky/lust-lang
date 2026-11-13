// The items of `v.iter()` over a local `Vec<&'a str>` hold `'a`, not the
// borrow of `v`, so copying them out outlives `v` (rustc accepts each).
fn push_all<'a>(src: &[&'a str], keep: &mut Vec<&'a str>) {
    let v: Vec<&'a str> = src.to_vec();
    for x in v.iter() {
        keep.push(*x);
    }
}
fn collect_copied<'a>(src: &[&'a str]) -> Vec<&'a str> {
    let v: Vec<&'a str> = src.iter().copied().collect();
    let out: Vec<&'a str> = v.iter().copied().collect();
    out
}
fn first_long<'a>(src: &[&'a str]) -> Option<&'a str> {
    let v: Vec<&'a str> = src.to_vec();
    let mut it = v.iter().filter(|s| s.len() > 1);
    match it.next() {
        Some(s) => Some(*s),
        None => None,
    }
}
fn pairs<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let x: Vec<&'a str> = a.to_vec();
    let y: Vec<&'a str> = b.to_vec();
    let mut out = Vec::new();
    for (p, q) in x.iter().zip(y.iter()) {
        out.push(*p);
        out.push(*q);
    }
    for s in x.iter().chain(y.iter()).rev() {
        out.push(*s);
    }
    out
}
fn main() {
    let s = String::from("hello");
    let t = String::from("w");
    let src = [s.as_str(), t.as_str()];
    let mut keep = Vec::new();
    push_all(&src, &mut keep);
    println!("{:?}", keep);
    println!("{:?}", collect_copied(&src));
    println!("{:?}", first_long(&src));
    println!("{:?}", pairs(&src, &src));
    let mut g: Vec<&'static str> = Vec::new();
    let v: Vec<&'static str> = vec!["a", "b"];
    for x in v.iter() {
        g.push(x);
    }
    drop(v);
    println!("{:?}", g);
}
