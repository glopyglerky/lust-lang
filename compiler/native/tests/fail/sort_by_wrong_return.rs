// expect: L0110
fn main() {
    let mut v = vec![2, 1];
    v.sort_by(|a, b| a < b);
    println!("{:?}", v);
}
