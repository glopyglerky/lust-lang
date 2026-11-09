// expect: L0110
fn main() {
    let v = vec![1, 2];
    let w: Vec<i32> = v.iter().collect();
    println!("{:?}", w);
}
