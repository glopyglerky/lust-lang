// expect: L0612
fn main() {
    let v = vec![1, 2];
    let s: String = v.iter().collect();
    println!("{}", s);
}
