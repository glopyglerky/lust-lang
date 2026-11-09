// expect: L0610
// rustc E0283: nothing fixes the result type of `sum`.
fn main() {
    let v = vec![1u64, 2];
    let s = v.iter().sum();
    println!("{}", s);
}
