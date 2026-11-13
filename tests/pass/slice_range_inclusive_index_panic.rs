// panic: slice index starts at 2 but ends at 1
// A backwards `RangeInclusive` value used as an index (tf-slice).
fn main() {
    let n = std::env::args().count();
    let v = vec![10, 20, 30];
    let r = 2..=n - 1;
    println!("{:?}", &v[r]);
}
