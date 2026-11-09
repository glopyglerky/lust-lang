// expect: L0303
fn main() {
    let mut v = vec![2, 1];
    let first = &v[0];
    v.sort();
    println!("{} {:?}", first, v);
}
