// An impl-item macro whose expansion is `#[attr] $vis fn ..` followed by
// another macro call (ts-rust data_accessor!): the expansion is items.
#[derive(Clone, Copy)]
struct N(u32);
macro_rules! acc {
    ($(#[$attr:meta])* $vis:vis fn $name:ident, $name_in:ident($n:ident) -> $ty:ty { $def:expr }) => {
        $(#[$attr])*
        #[must_use]
        $vis fn $name(self) -> $ty { let $n: N = self; $n.0 + $def }
        #[doc = concat!("`N::", stringify!($name), "` on d.")]
        #[must_use]
        $vis fn $name_in(self, d: u32) -> $ty { self.$name() + d }
    };
}
impl N {
    acc! { /// doc
        pub fn a, a_in(n) -> u32 { 1 } }
    acc! { fn b, b_in(n) -> u32 { 2 } }
}
fn main() { let n = N(3); println!("{} {} {} {}", n.a(), n.a_in(1), n.b(), n.b_in(2)); }
