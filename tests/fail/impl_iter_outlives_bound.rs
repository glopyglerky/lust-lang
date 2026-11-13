// expect: L0733
// rustc: lifetime may not live long enough. `+ 'b` needs the hidden
// type's `'a` borrow to outlive `'b`.
struct S {
    w: Vec<String>,
}
fn mk<'a, 'b>(a: &'a S, b: &'b S) -> impl Iterator<Item = &'a String> + 'b {
    let v: Vec<&'a String> = a.w.iter().collect();
    v.into_iter()
}
fn main() {
    let s1 = S { w: vec!["p".into()] };
    let s2 = S { w: vec!["q".into()] };
    for x in mk(&s1, &s2) {
        println!("{:?}", x);
    }
}
