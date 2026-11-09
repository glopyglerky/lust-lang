// expect: L0304
fn main() {
    let mut s = String::new();
    let a = &mut s;
    let b = &mut s;
    a.push('x');
    b.push('y');
}
