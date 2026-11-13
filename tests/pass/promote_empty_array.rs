// `&[]` and `&mut []` are promoted to `'static`, as in rustc: they may be
// returned, kept past their statement, and given any element type.
fn empty() -> &'static [u8] {
    &[]
}
fn empty_mut() -> &'static mut [u8] {
    &mut []
}
fn pick<'a>(v: &'a [u8], n: u8) -> &'a [u8] {
    match n {
        0 => v,
        _ => &[],
    }
}
fn or_empty(o: Option<&[u8]>) -> &[u8] {
    o.unwrap_or(&[])
}
fn no_strings() -> &'static [String] {
    &[]
}
fn main() {
    let v = vec![1u8, 2];
    let kept = {
        let e: &[u8] = &[];
        e
    };
    let a = empty_mut();
    let b = empty_mut();
    println!("{} {} {} {}", empty().len(), pick(&v, 0).len(), pick(&v, 1).len(), kept.len());
    println!("{} {} {} {}", or_empty(None).len(), or_empty(Some(&v)).len(), a.len() + b.len(), no_strings().len());
}
