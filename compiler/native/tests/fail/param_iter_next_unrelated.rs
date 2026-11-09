// expect: L0733
// rustc: lifetime may not live long enough. The item of `it.next()` is `&'a u32`, and
// nothing says `'a: 'b`.
fn pull<'a, 'b, I: Iterator<Item = &'a u32>>(mut it: I, out: &mut Vec<&'b u32>) {
    while let Some(x) = it.next() {
        out.push(x);
    }
}
fn main() {
    let n = [1u32];
    let mut v = Vec::new();
    pull(n.iter(), &mut v);
    println!("{}", v.len());
}
