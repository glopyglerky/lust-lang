// expect: L0303
fn main() {
    let mut v = vec![1, 2, 3];
    let mut n = 0;
    let items: Vec<&i32> = v.iter().filter(|_| { n += 1; true }).collect();
    v.push(4);
    println!("{:?} {}", items, n);
}
