// expect: L0110
// rustc E0308: `Vec::<u8>::len` names `Vec<u8>`'s method; a `String`
// does not coerce to `&Vec<u8>`.
fn main() {
    let s = String::from("ab");
    println!("{}", Vec::<u8>::len(&s));
}
