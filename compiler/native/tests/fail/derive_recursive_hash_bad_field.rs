// expect: L0110
// rustc E0277: the recursive field is fine, but `Bad` has no Hash
// impl, so the derived Hash body does not type check. The
// coinductive cycle for `S` itself must not hide the bad field.
struct Bad;
#[derive(Hash)]
struct S { p: Option<Box<S>>, b: Bad }
fn main() { println!("x"); }
