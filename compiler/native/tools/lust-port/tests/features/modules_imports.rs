// feature: modules.imports
// form: inline mods with a glob import, a re-export, a renamed import and a trait imported as _
mod inner {
    pub fn a() -> i32 { 1 }
    pub fn b() -> i32 { 2 }
    pub trait Greet { fn greet(&self) -> i32; }
    impl Greet for i32 { fn greet(&self) -> i32 { *self + 40 } }
}
mod api { pub use super::inner::a as first; }
use inner::*;
use inner::b as second;
use inner::Greet as _;
fn main() { println!("{} {} {} {}", a(), second(), api::first(), 2.greet()); }
