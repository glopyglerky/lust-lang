// expect: L0110
// rustc E0308: `map`'s `U` is not known in the closure body, so `&v`
// stays `&&Vec<u8>` and the result is no `Result<&Vec<u8>, ()>`.
fn g<'a>(v: &'a Vec<u8>) -> Result<&'a Vec<u8>, ()> {
    Ok::<u8, ()>(1).map(|_k| &v)
}
fn main() {
    let v = vec![1u8];
    println!("{:?}", g(&v));
}
