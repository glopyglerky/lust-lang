// feature: lifetimes.named
// form: an explicit &'a self receiver that returns the borrowed field
struct Ctx<'a> { text: &'a str }
impl<'a> Ctx<'a> {
    fn text(&'a self) -> &'a str { self.text }
}
fn main() { let c = Ctx { text: "hi" }; println!("{}", c.text()); }
