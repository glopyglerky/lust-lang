// expect: L0110
// rustc E0747: a char const in a user struct's type-param slot.
struct S<T>(Option<T>);
fn main() { let _x: S<'c'> = S(None); }
