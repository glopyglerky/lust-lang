// expect: L0734
// Sol review probe `trait_method_param`; rustc: E0308 method not compatible with trait
trait Read { fn get<'a>(&self, x: &'a i32) -> &'a i32; }
struct S;
impl Read for S {
    fn get(&self, x: &'static i32) -> &'static i32 { x }
}
fn main() {}
