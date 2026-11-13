// A fn item as a `&dyn Fn` argument in a program with no generics and
// no closures (mono still builds the vtable).
fn apply(f: &dyn Fn(u8) -> u8) -> u8 {
    f(2)
}
fn double(x: u8) -> u8 {
    x * 2
}
fn main() {
    println!("{}", apply(&double));
}
