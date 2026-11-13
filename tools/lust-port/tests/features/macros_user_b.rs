// feature: macros.user
// form: ty fragment macro that writes a struct, and a recursive tt counter
macro_rules! wrapper { ($name:ident, $t:ty) => { struct $name { v: $t } }; }
macro_rules! count { () => { 0 }; ($h:tt $($t:tt)*) => { 1 + count!($($t)*) }; }
wrapper!(Age, i32);
fn main() { let a = Age { v: 4 }; println!("{} {}", a.v, count!(x y z)); }
