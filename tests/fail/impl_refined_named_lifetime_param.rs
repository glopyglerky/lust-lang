// expect: L0306
// rustc E0597: the trait ties the result to both arguments by `'x`;
// the impl's looser `_b: &str` does not free `b` for callers.
trait P { fn pick<'x>(&self, a: &'x str, b: &'x str) -> &'x str; }
struct S;
impl P for S { fn pick<'x>(&self, a: &'x str, _b: &str) -> &'x str { a } }
fn main() {
    let a = String::from("a");
    let r;
    {
        let b = String::from("b");
        r = S.pick(&a, &b);
    }
    println!("{}", r);
}
