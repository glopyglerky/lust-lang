// expect: L0303
fn main() {
    let mut v = vec![1, 2, 3];
    let r = &v;
    let items: Vec<&i32> = r.iter().filter(|x| **x < r.len() as i32).collect();
    v.push(4);
    println!("{:?}", items);
}
