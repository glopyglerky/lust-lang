// Derived Ord and slice ordering over 128-bit integers (the cmp glue
// compares them through the runtime helpers).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct W { a: i128, b: u128 }
fn main() {
    let v = vec![W { a: -5, b: 1 }, W { a: -5, b: u128::MAX }, W { a: i128::MIN, b: 0 }, W { a: 3, b: 2 }];
    for x in &v { for y in &v { print!("{:?} ", x.cmp(y)); } println!(); }
    println!("{} {:?}", vec![1u128, 2] < vec![1u128, 3], [i128::MIN, 0].partial_cmp(&[i128::MAX, 0]));
}
