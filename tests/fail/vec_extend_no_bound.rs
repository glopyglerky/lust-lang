// expect: L0112
fn ext<T>(v: &mut Vec<T>, s: &[T]) {
    v.extend_from_slice(s);
}
fn main() {
    let mut v = vec![1];
    ext(&mut v, &[2]);
    println!("{:?}", v);
}
