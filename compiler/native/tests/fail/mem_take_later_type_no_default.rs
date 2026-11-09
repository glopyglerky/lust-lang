// expect: L0612
// rustc E0277: the type a later use fixes has no `Default`.
#[derive(Debug)]
struct NoDef(u8);
fn main() {
    let mut x = None;
    let r = match x.as_mut() {
        Some(p) => std::mem::take(p),
        None => NoDef(1),
    };
    x = Some(r);
    println!("{:?}", x);
}
