// expect: L0733
// `sort_by_key`'s bound is `FnMut(&T) -> K` with a higher-ranked
// `&T`, so the key cannot borrow from the element (rustc:
// lifetime may not live long enough).
fn main() {
    let mut v = vec![(2, 1), (1, 2)];
    v.sort_by_key(|p| &p.0);
    println!("{:?}", v);
}
