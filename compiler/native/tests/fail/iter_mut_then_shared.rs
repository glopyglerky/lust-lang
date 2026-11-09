// expect: L0303
fn main() {
    let mut w = vec![1, 2];
    let it = w.iter_mut();
    let r = &w;
    for x in it {
        *x += 1;
    }
    println!("{:?}", r);
}
