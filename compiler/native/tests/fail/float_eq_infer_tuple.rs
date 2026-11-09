// expect: L0612
// rustc E0277: an unannotated float tuple does not satisfy `T: Eq`.
fn e<T: Eq>(_: T) {}
fn main() { e((1.5, 2.5)); }
