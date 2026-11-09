// expect: L0306
// rustc E0597: the flattened iterator borrows the Vec of Vecs.
fn main() {
    let it;
    {
        let v = vec![vec![1, 2]];
        it = v.iter().flatten();
    }
    println!("{}", it.count());
}
