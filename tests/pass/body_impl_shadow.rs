// A block type with a block impl shadows a module type of the same name.
struct S(u8);
impl S { fn who(&self) -> &'static str { "outer" } }
fn f() {
    struct S(u16);
    impl S { fn who(&self) -> &'static str { "inner" } }
    println!("{}", S(1).who());
}
fn main() { f(); println!("{}", S(2).who()); }
