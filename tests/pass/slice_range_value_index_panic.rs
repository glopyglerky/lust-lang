// panic: range end index 5 out of range for slice of length 3
// A `RangeTo` value used as an index checks like the syntactic
// form, at the brackets (tf-slice).
fn main() {
    let n = std::env::args().count();
    let v = vec![10, 20, 30];
    let r = ..n + 4;
    println!("{:?}", &v[r]);
}
