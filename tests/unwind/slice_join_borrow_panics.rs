// A written Borrow<str> that panics, or grows between its two calls, in
// join/concat: the panic leaves at once (no later borrow runs), as core's.
use std::borrow::Borrow;
use std::cell::Cell;
struct P(&'static str, Cell<u32>, u32);
impl Borrow<str> for P {
    fn borrow(&self) -> &str {
        self.1.set(self.1.get() + 1);
        println!("borrow {} #{}", self.0, self.1.get());
        if self.1.get() == self.2 {
            panic!("boom");
        }
        self.0
    }
}
struct Grow(Cell<u32>);
impl Borrow<str> for Grow {
    fn borrow(&self) -> &str {
        self.0.set(self.0.get() + 1);
        if self.0.get() == 1 { "a" } else { "abcdef" }
    }
}
fn main() {
    std::panic::set_hook(Box::new(|i| println!("panic: {}", i.payload_as_str().unwrap_or("?"))));
    let p = [P("a", Cell::new(0), 9), P("b", Cell::new(0), 2), P("c", Cell::new(0), 9)];
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.join("+")));
    println!("{}", r.is_err());
    let g = [Grow(Cell::new(0)), Grow(Cell::new(0)), Grow(Cell::new(0))];
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| g.join(",")));
    println!("{}", r.is_err());
    let g = [Grow(Cell::new(0)), Grow(Cell::new(0))];
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| g.concat()));
    println!("{:?}", r.ok());
}
