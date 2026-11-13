// expect: L0612
fn main() {
    let v = vec![String::new()];
    let c: Vec<String> = v.iter().copied().collect();
    println!("{:?}", c);
}
