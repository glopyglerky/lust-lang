// expect: L0110
fn main() {
    let r: Result<Vec<i32>, ()> = vec![1, 2].into_iter().collect();
    println!("{:?}", r);
}
