// expect: L0303
fn main() {
    let mut v = vec![1, 2];
    let r = &mut v[v.len() - 1];
    *r = 9;
    println!("{}", v[1]);
}
