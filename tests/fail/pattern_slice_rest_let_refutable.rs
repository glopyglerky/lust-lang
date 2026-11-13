// expect: L0110
fn last(v: &[u8]) -> u8 {
    let [.., a] = v;
    *a
}
fn main() {
    println!("{}", last(&[1, 2, 3]));
}
