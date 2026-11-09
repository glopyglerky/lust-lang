// expect: L0733
// rustc: lifetime may not live long enough. `'c` is invariant behind
// `&'a mut TypeParser<'c>`, so it cannot shrink to `'a`.
struct TypeParser<'c> {
    src: &'c str,
}

fn shrink<'a, 'c: 'a>(p: &'a mut TypeParser<'c>) -> &'a mut TypeParser<'a> {
    p
}

fn main() {
    let s = String::from("x");
    let mut tp = TypeParser { src: &s };
    let q = shrink(&mut tp);
    println!("{}", q.src);
}
