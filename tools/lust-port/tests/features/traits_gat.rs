// feature: traits.gat
// form: generic associated type with a lifetime parameter
trait Lender { type Item<'a> where Self: 'a; fn next<'a>(&'a mut self) -> Self::Item<'a>; }
struct One { done: bool }
impl Lender for One {
    type Item<'a> = &'a str;
    fn next<'a>(&'a mut self) -> Self::Item<'a> { self.done = true; "x" }
}
fn main() { let mut o = One { done: false }; println!("{}", o.next()); }
