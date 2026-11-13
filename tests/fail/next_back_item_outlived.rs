// expect: L0306
// rustc E0597: the item from next_back borrows the Vec.
fn main() {
    let x;
    {
        let v = vec![1, 2];
        let mut it = v.iter();
        x = it.next_back();
    }
    println!("{x:?}");
}
