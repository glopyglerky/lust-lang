// expect: L0641
// Vec::splice as a value (its removed items collected): lust takes
// `v.splice(range, items);` as an expression statement only, where the
// Splice drops at once.
fn main() {
    let mut v = vec![1, 2, 3];
    let r: Vec<i32> = v.splice(1..2, [9]).collect();
    println!("{r:?} {v:?}");
}
