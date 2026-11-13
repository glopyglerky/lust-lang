// `Self::f()` in a trait default body calls the trait fn the template
// check picked, also when the impl type has an inherent fn of that name.
struct S;
impl S { fn mk() -> u8 { 50 } }
trait D { fn mk() -> u8; fn run(&self) -> u8 { Self::mk() } }
impl D for S { fn mk() -> u8 { 5 } }
struct T2;
impl D for T2 { fn mk() -> u8 { 7 } }
trait E { fn mk2() -> u8 { 1 } fn go(&self) -> u8 { Self::mk2() + 1 } }
impl E for S {}
fn main() { println!("{} {} {} {}", S.run(), S::mk(), T2.run(), S.go()); }
