// mem::take on any type whose Default is derived or core's, through the
// default glue: structs, enums with a #[default] variant, tuples.
#[derive(Default, Debug)]
struct S { a: u8, b: String, v: Vec<u8> }
#[derive(Default, Debug)]
enum E { #[default] A, B(u8) }
fn main() {
    let mut s = S { a: 1, b: String::from("x"), v: vec![1] };
    let t = std::mem::take(&mut s);
    println!("{:?} {:?}", s, t);
    let mut e = E::B(3);
    println!("{:?} {:?}", std::mem::take(&mut e), e);
    let mut tup = (1u8, String::from("y"));
    println!("{:?} {:?}", std::mem::take(&mut tup), tup);
}
