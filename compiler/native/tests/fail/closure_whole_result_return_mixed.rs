// expect: L0110
// rustc E0308: the closure's own return type is the first `return`'s
// `&&Box<u8>`; the tail `&u8` does not coerce to it.
fn g<'a>(v: &'a Vec<u8>, b: &'a Box<u8>) -> &'a u8 {
    Some(1u8).map_or_else(|| panic!(), |k| {
        if k == 7 {
            return &b;
        }
        &v[0]
    })
}
fn main() {
    let v = vec![1u8];
    let b = Box::new(2u8);
    println!("{}", g(&v, &b));
}
