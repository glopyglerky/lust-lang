// expect: L0610
fn main() {
    let v = vec![1, 2];
    let c = v.iter().collect();
    println!("{:?}", c);
}
