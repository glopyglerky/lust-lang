// panic: range end index 9 out of range for slice of length 3
// splice checks its range as drain does, at core's slice::range sites (R4).
fn main() {
    let n = std::env::args().count();
    let mut v = vec![1, 2];
    v.splice(n..n, [5]);
    println!("{v:?}");
    v.splice(n..9, [5]);
}
