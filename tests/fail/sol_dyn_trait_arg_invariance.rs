// expect: L0203
// Sol review probe `dyn_trait_arg_invariance`; rustc: lifetime may not live long enough
trait Read<T> {}
fn bad<'a, 'b: 'a>(x: &'a dyn Read<&'b i32>) -> &'a dyn Read<&'a i32> { x }
fn main() {}
