// expect: L0640
fn main() {
    let r: Result<i32, ()> = vec![Ok(1)].into_iter().collect();
    println!("{:?}", r);
}
