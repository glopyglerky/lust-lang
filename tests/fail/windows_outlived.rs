// expect: L0306
// rustc E0597: the windows borrow the Vec.
fn main() {
    let w;
    {
        let v = vec![1, 2, 3];
        w = v.windows(2);
    }
    println!("{:?}", w.count());
}
