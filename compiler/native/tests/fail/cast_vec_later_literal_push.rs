// expect: L0116
// rustc E0605: the cast is checked at body end, after `v.push(5)` made
// the element a literal that falls back to `i32`.
fn main() {
    let mut v = Vec::new();
    let w = &v as &Vec<u8>;
    println!("{:?}", w);
    v.push(5);
}
