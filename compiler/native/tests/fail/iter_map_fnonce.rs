// expect: L0612
fn main() {
    let v = vec![1, 2];
    let s = String::new();
    let w: Vec<String> = v.iter().map(move |_| s).collect();
    println!("{:?}", w);
}
