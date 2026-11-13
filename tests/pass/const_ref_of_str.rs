// `&"ab"` in a const or static is a `&&str` (lust took it as the `&str`); `&*S` is the `&str`.
const C: &&str = &"ab";
const D: &&str = &("ab");
const S: &str = "xyz";
const T: &str = &*S;
const U: &&str = &S;
static V: &&str = &"ab";
fn main() {
    println!("{} {} {} {} {}", C, D, T, U, V);
    println!("{} {}", C.as_ptr() == "ab".as_ptr(), V.as_ptr() == C.as_ptr());
}
