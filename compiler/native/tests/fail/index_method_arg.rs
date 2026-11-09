// expect: L0303
fn main() {
    let mut v = vec![vec![1]];
    v[0].push(v[0].len() as i32);
    println!("{}", v[0][1]);
}
