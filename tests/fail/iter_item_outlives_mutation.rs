// expect: L0303
fn main() {
    let mut v = vec![1, 2, 3];
    let first = v.iter().next();
    v.push(9);
    println!("{:?}", first);
}
