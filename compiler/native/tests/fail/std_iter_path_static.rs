// expect: L0733
// rustc: lifetime may not live long enough (the iterator borrows `v`, not 'static).
fn bad(v: &Vec<u8>) -> std::slice::Iter<'static, u8> {
    v.iter()
}

fn main() {
    let v = vec![1u8];
    println!("{}", bad(&v).count());
}
