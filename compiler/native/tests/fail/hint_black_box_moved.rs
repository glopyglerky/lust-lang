// expect: L0301
// rustc E0382: black_box takes its argument by value.
fn main() {
    let v = vec![1];
    let w = std::hint::black_box(v);
    println!("{:?} {:?}", v, w);
}
