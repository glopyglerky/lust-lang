// expect: L0104
// rustc E0277: `(&&s).clone()` is `&String` (the by-value step of `&&String`
// clones the reference), and there is no `From<&String>` for `Vec<u8>`.
#![allow(noop_method_call, suspicious_double_ref_op)]
fn main() {
    let s = String::from("s");
    let o: Vec<u8> = (&&s).clone().into();
    println!("{:?}", o);
}
