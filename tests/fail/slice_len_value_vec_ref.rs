// expect: L0612
// rustc E0631: a fn item takes no deref coercion: `<[u32]>::len` cannot take `&Vec<u32>`.
fn main() {
    let v = vec![1u32];
    println!("{:?}", Some(&v).map(<[u32]>::len));
}
