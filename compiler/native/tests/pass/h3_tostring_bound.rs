// ToString as a bound: core's blanket impl over Display, a user impl for a
// type without Display, and `t.to_string()` under a `T: Display` bound.
use std::fmt;
struct D(i32);
impl fmt::Display for D { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "D{}", self.0) } }
struct U;
impl ToString for U { fn to_string(&self) -> String { "U!".to_string() } }
trait Tag { fn tag(&self) -> String; }
impl<T: ToString> Tag for T { fn tag(&self) -> String { self.to_string() } }
fn show(x: impl ToString) -> String { x.to_string() }
fn by_ref<T: ToString + ?Sized>(x: &T) -> String { x.to_string() }
fn disp<T: fmt::Display>(t: T) -> String { t.to_string() }
fn main() {
    println!("{} {} {} {}", 5.tag(), "x".tag(), D(1).tag(), U.tag());
    println!("{} {} {} {}", show(3), show(D(2)), by_ref("s"), by_ref(&U));
    println!("{} {} {}", disp(4), disp(D(3)), more());
}
fn path_call<T: ToString + ?Sized>(t: &T) -> String { ToString::to_string(t) }
#[allow(dead_code)]
fn more() -> String { path_call("str") + &path_call(&7) }
