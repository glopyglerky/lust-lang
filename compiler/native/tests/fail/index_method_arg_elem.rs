// expect: L0303
fn main() {
    let mut v = vec![vec![5]];
    v[0].push(v[0][0]);
    println!("{}", v[0][1]);
}
