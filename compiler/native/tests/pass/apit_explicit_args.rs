// Explicit args name only the declared params; `impl Trait` args stay
// inferred, in calls, method calls, and fn values.
use std::fmt::Display;
fn two<T: Display>(a: T, b: impl Display) -> String {
    format!("{}{}", a, b)
}
fn main0() {
    println!("{}", two::<u8>(1, 2));
    let f = two::<u8>;
    println!("{}", f(3, "x"));
}

struct S;
impl S {
    fn m(&self, x: impl Display) -> String {
        format!("[{}]", x)
    }
    fn n<T: Display>(&self, a: T, x: impl Display) -> String {
        format!("[{}{}]", a, x)
    }
}
fn main() {
    main0();
    println!("{} {}", S.m(1u8), S.n::<u8>(2, 3));
}
