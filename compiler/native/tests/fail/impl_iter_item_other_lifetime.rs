// expect: L0733
// rustc: lifetime may not live long enough. The items are `'a` but the
// closure returns `b`'s borrow.
struct S {
    w: Vec<String>,
}
fn mk<'a, 'b>(a: &'a S, b: &'b S) -> impl Iterator<Item = &'a String> {
    [0usize, 1].into_iter().map(move |_n| &b.w[0])
}
fn main() {
    let s1 = S { w: vec!["p".into()] };
    let s2 = S { w: vec!["q".into()] };
    for x in mk(&s1, &s2) {
        println!("{:?}", x);
    }
}
