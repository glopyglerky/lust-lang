// mem::take of a place whose type a later use fixes: rustc keeps
// `T: Default` pending until then, so lust checks it at body end.
#[derive(Default, Debug)]
struct D(u8);
fn main() {
    let mut x = Default::default();
    let y = std::mem::take(&mut x);
    let z: String = x;
    println!("{:?} {:?}", y, z);
    let mut o = None;
    let r = match o.as_mut() {
        Some(p) => std::mem::take(p),
        None => Default::default(),
    };
    o = Some(r);
    let d: Option<D> = o;
    println!("{:?}", d);
}
