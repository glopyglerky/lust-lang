// expect: L0110
// rustc E0308: `(&&s).clone()` is `&String`, not `String`.
#![allow(noop_method_call, suspicious_double_ref_op)]
fn main() {
    let s = String::from("s");
    let v = &&s;
    let o: String = v.clone();
    println!("{}", o);
}
