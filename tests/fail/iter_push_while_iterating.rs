// expect: L0303
fn main() {
    let mut v = vec![1, 2, 3];
    for x in v.iter() {
        v.push(*x);
    }
}
