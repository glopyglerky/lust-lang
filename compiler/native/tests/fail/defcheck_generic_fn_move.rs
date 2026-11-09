// expect: L0301
// rustc E0382: a never-called generic fn is borrow checked
// as a template.
fn bad<T>(x: T) {
    let y = x;
    let z = x;
    let _ = (y, z);
}
fn main() {}
