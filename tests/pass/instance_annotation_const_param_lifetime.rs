// A mono instance's body annotations that name the template's const
// parameter (`[u8; N]`) or a named lifetime (`&'a T`) resolve as in the
// template (they were L0110 and L0203 at build after a passing check).
fn arr<const N: usize>() -> usize {
    let a: [u8; N] = [0; N];
    a.len()
}
fn get<'a, T: Copy>(w: &'a T) -> T {
    let x: &'a T = w;
    *x
}
struct W<'a, T> {
    r: &'a T,
}
fn field<'a, T: Copy>(w: W<'a, T>) -> T {
    let x: &'a T = w.r;
    *x
}
fn main() {
    println!("{} {} {} {}", arr::<3>(), arr::<0>(), get(&7u8), field(W { r: &'x' }));
}
