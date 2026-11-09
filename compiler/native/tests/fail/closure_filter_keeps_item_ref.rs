// expect: L0733
// rustc E0521: `filter`'s `&item` is higher-ranked; storing that outer
// reference escapes the closure.
fn main() {
    let v = vec![1, 2, 3];
    let mut keep: Vec<&&i32> = Vec::new();
    let n = v.iter().filter(|x| { keep.push(x); true }).count();
    println!("{} {}", n, keep.len());
}
