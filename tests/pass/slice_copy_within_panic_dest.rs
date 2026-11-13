// panic: dest is out of bounds
// `copy_within` past the end panics at the call (tf-slice).
fn main() {
    let n = std::env::args().count();
    let mut v = vec![0, 1, 2];
    println!("start {n}");
    v.copy_within(0..2, 2);
    println!("{:?}", v);
}
