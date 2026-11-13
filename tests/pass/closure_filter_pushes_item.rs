// `filter` passes `&item`: the outer borrow is higher-ranked, the item free.
fn main() {
    let v = vec![1, 2, 3];
    let mut keep: Vec<&i32> = Vec::new();
    let n = v.iter().filter(|x| { keep.push(*x); true }).count();
    println!("{} {}", n, keep.len());
}
