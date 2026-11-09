// expect: L0110
// rustc E0308: the turbofish fixes the element type (it was ignored).
fn main() {
    let x = Option::<u8>::None;
    let y: Option<u16> = x;
    let z: Option<u16> = Some::<u8>(3);
    let r: Result<u16, ()> = Ok::<u8, _>(1);
    println!("{:?} {:?} {:?}", y, z, r);
}
