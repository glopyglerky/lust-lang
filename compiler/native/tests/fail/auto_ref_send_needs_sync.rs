// expect: L0612
// rustc E0277: &T is Send only when T is Sync; T: Send is not enough.
fn need<T: Send>(_t: &T) {}
fn g<T: Send>(t: &T) { need(&t); }
fn main() { g(&5u8); }
