// expect: L0301
// A Drain is not Copy.
fn main() {
    let mut v = vec![1, 2, 3, 4];
    let d = v.drain(..);
    let _e = d;
    println!("{}", d.len());
}
