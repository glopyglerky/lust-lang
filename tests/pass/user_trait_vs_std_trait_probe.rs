// A user trait's method against a std trait's of the same name, placed on
// rustc's probe order (by value, `&`, `&mut` at each step): the derived
// `Clone::clone` (`&self`) comes before a user `fn clone(&mut self)`, a
// user `fn eq(self, ..)` comes before `PartialEq::eq` (`&self`), and a
// blanket std impl whose bounds fail (`ToString` without `Display`) is no
// candidate at all.
#[derive(Clone, PartialEq, Debug)]
struct S(i32);
trait Sh {
    fn clone(&mut self) -> &'static str;
    fn eq(self, o: &S) -> &'static str;
}
impl Sh for S {
    fn clone(&mut self) -> &'static str {
        "user clone"
    }
    fn eq(self, _o: &S) -> &'static str {
        "user eq"
    }
}
struct N(i32);
trait Named {
    fn to_string(&self) -> String;
    fn into_iter(self) -> i32;
    fn to_owned(&self) -> i32;
}
impl Named for N {
    fn to_string(&self) -> String {
        format!("N{}", self.0)
    }
    fn into_iter(self) -> i32 {
        self.0 + 1
    }
    fn to_owned(&self) -> i32 {
        self.0 + 2
    }
}
fn main() {
    let mut s = S(1);
    let c = s.clone();
    println!("{:?}", c);
    println!("{}", s.eq(&S(1)));
    let n = N(5);
    println!("{} {}", n.to_string(), n.to_owned());
    println!("{}", n.into_iter());
}
