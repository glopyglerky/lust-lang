// expect: L0612
// rustc E0631: the TryInto error of Vec<u8> to [u8; 2] is Vec<u8>, not String.
fn vec2(v: Vec<u8>) -> Result<[u8; 2], String> {
    let a = v.try_into().map_err(|items: String| items)?;
    Ok(a)
}
fn main() {
    println!("{:?}", vec2(vec![1]));
}
