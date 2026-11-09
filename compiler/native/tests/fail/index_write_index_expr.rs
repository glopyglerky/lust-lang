// expect: L0303
fn main() {
    let mut v = vec![1, 2];
    v[v.len() - 1] = 9;
    println!("{}", v[1]);
}
