// expect: L0640
fn main() {
    let v = vec![1, 2];
    let mut it = v.iter().map(|x| x + 1);
    println!("{:?}", it.peek());
}
