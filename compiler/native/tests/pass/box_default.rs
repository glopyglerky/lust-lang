// `Box::default()`, `Vec::default()`, and kin with no turbofish on
// the type segment: the expected type names the builtin Self, as in
// rustc (W-BENCH3 bug 16).
#[derive(Default, Debug)]
struct L {
    n: u32,
}
fn main() {
    let b: Box<L> = Box::default();
    let v: Vec<u8> = Vec::default();
    let o: Option<u8> = Option::default();
    let s: String = String::default();
    println!("{:?} {:?} {:?} {:?}", b, v, o, s);
    let b2: Box<L> = Box::<L>::default();
    println!("{:?}", b2);
}
