// expect: L0631
// rustc E0658: `first` is a slice method, so the call goes through
// `Vec`'s `Deref`, which is not const.
const fn f(v: &Vec<u8>) -> Option<&u8> {
    v.first()
}
fn main() {
    println!("{:?}", f(&vec![1]));
}
