// expect: L0307
// rustc E0515: the iterator borrows a local that the fn drops.
fn dangle() -> std::slice::Iter<'static, u8> {
    let v = vec![1u8];
    let it = v.iter();
    it
}

fn main() {
    println!("{}", dangle().count());
}
