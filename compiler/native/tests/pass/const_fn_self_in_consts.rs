// `Self` inside a const fn body that a const initializer runs is the
// fn's impl type (`const fn new() -> Self { Self { .. } }`), also
// through `Self::f(..)` calls.
#[derive(Debug, Clone, Copy)]
struct Inner {
    v: u16,
}
impl Inner {
    const fn new(v: u16) -> Self {
        Self { v }
    }
    const fn twice(self) -> Self {
        Self::new(self.v * 2)
    }
}
#[derive(Debug)]
struct Outer {
    a: Inner,
    b: Inner,
}
impl Outer {
    const fn make(x: u16) -> Self {
        Self { a: Inner::new(x), b: Inner::new(x + 1) }
    }
    const DEFAULT: Outer = Self::make(5);
    const OTHER: Self = Outer::make(Self::BASE);
    const BASE: u16 = 9;
}
const O: Outer = Outer::make(1);
const I: Inner = Inner::new(3);
fn main() {
    println!("{:?} {:?} {:?} {:?}", O, Outer::DEFAULT, Outer::OTHER, I);
}
