// expect: L0733
// rustc: lifetime may not live long enough. The closure returns
// `&mut Vec<&'a str>`, which is invariant in `'a`: `map`'s `U` cannot be
// `&mut Vec<&'b str>`.
pub struct S<'a> {
    v: Vec<&'a str>,
}
fn f<'a, 'b>(s: &'b mut S<'a>) -> Option<&'b mut Vec<&'b str>> {
    Some(s).map(|st| &mut st.v)
}
fn main() {
    let x = String::from("x");
    let mut s = S { v: vec![x.as_str()] };
    f(&mut s).unwrap().push("y");
    println!("{:?}", s.v);
}
