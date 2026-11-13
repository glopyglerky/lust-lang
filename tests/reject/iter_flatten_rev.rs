// expect: L0612
// Valid Rust: Flatten is double-ended when its inner iterators are;
// lust runs it forward only so far.
fn main() {
    let v = vec![vec![1, 2], vec![3]];
    let r: Vec<i32> = v.iter().flatten().rev().copied().collect();
    println!("{r:?}");
}
