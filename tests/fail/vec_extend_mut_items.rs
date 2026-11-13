// expect: L0110
// rustc E0277: Vec<u32> does not implement Extend<&mut u32>.
fn main() {
    let mut v: Vec<u32> = Vec::new();
    let mut w = vec![1u32, 2];
    v.extend(w.iter_mut());
    println!("{:?} {:?}", v, w);
}
