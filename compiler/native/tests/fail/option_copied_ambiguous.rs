// expect: L0610
// rustc E0034: with the payload not known yet, both `Option<&T>::copied`
// and `Option<&mut T>::copied` fit the receiver.
fn main() {
    let mut x = None;
    let y = x.copied();
    let v = 5;
    x = Some(&v);
    println!("{:?} {:?}", x, y);
}
