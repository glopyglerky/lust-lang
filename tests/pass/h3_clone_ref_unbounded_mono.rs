// `t.clone()` on `t: &T` clones the reference when T has no Clone bound, also
// in an instance where T is Clone (i32, S); with `T: Clone` it clones the T.
fn g<'a, T>(t: &'a T) -> &'a T { t.clone() }
fn h<T: Clone>(t: &T) -> T { t.clone() }
#[derive(Clone, Debug)] struct S(i32);
fn main() { let v = 5i32; println!("{} {} {:?} {:?}", g(&v), h(&v), h(&S(1)), g(&S(2))); println!("{:?}", h(&vec![1])); }
