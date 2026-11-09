// A call through an `Fn` bound on an impl's or trait's type param follows
// the bound's lifetimes: a lifetime the bound writes is the fn's own, an
// elided one is higher-ranked (fresh at each call). The bodies are
// checked as templates.
struct W<'a, F> {
    f: F,
    r: &'a u8,
}
impl<'a, F: Fn(&'a u8) -> &'a u8> W<'a, F> {
    fn get(&self) -> &'a u8 {
        (self.f)(self.r)
    }
}
impl<'a, F> W<'a, F>
where
    F: FnMut(&'a u8) -> &'a u8,
{
    fn get_mut(&mut self) -> &'a u8 {
        (self.f)(self.r)
    }
}
struct H<F> {
    f: F,
}
impl<F: Fn(&u8) -> &u8> H<F> {
    fn go(&self) -> u8 {
        let x = 5u8;
        *(self.f)(&x)
    }
    fn pass<'b>(&self, y: &'b u8) -> &'b u8 {
        (self.f)(y)
    }
}
trait Pick<F: Fn(&u8) -> &u8> {
    fn pick<'a>(&self, f: &F, x: &'a u8) -> &'a u8 {
        f(x)
    }
}
struct S;
impl<F: Fn(&u8) -> &u8> Pick<F> for S {}
fn main() {
    println!("templates only");
}
