// panic: range end index 9 out of range for slice of length 3
// extend_from_within checks its range with core's slice::range sites (R4).
fn main() {
    let n = std::env::args().count();
    let mut v = vec![String::from("a"), String::from("b"), String::from("c")];
    v.extend_from_within(n..);
    println!("{v:?}");
    let mut w = vec![1, 2, 3];
    w.extend_from_within(n..9);
}
