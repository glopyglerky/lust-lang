// `impl Trait` nested in argument types (`&impl Tr`, `&[impl Tr]`,
// `Vec<impl Tr>`, tuples, `&mut impl Iterator`) is a synthetic type
// param, as at the top level.
use std::fmt::Display;
fn a(x: &impl Display) -> String {
    format!("{}", x)
}
fn b(xs: &[impl AsRef<str>]) -> usize {
    xs.iter().map(|s| s.as_ref().len()).sum()
}
fn c(v: Vec<impl Display>, t: (impl Copy, u8)) -> usize {
    let _ = t;
    v.len()
}
fn d(m: &mut impl Iterator<Item = u8>) -> Option<u8> {
    m.next()
}
struct S;
impl S {
    fn e(&self, xs: &[impl Display]) -> String {
        let mut out = String::new();
        for x in xs {
            out.push_str(&format!("{},", x));
        }
        out
    }
}
fn main() {
    println!("{} {}", a(&3u8), b(&["ab", "c"]));
    println!("{} {}", b(&[String::from("xyz")]), c(vec![1, 2], ('a', 3)));
    let mut it = vec![9u8, 8].into_iter();
    println!("{:?} {}", d(&mut it), S.e(&[1.5, 2.0]));
}
