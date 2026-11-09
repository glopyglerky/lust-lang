// expect: L0641
// A written `std::iter::Map` whose function type is not a fn pointer is not supported yet.
fn wrap<'a, F: FnMut(&'a u8) -> u8>(v: &'a [u8], f: F) -> std::iter::Map<std::slice::Iter<'a, u8>, F> {
    v.iter().map(f)
}

fn main() {
    let v = vec![1u8, 2];
    let s: u8 = wrap(&v, |x| x + 1).sum();
    println!("{}", s);
}
