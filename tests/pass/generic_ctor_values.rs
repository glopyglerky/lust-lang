// Generic tuple structs and tuple variants as values: inferred or
// turbofish args, iterator adapters, Fn bounds, generic callers, LUB.
#[derive(Debug)]
struct W<T>(T);
#[derive(Debug)]
enum E<T> {
    A(T),
    B(T, u8),
}
#[derive(Debug, Clone, Copy)]
struct P<T: Copy>(T, T);
fn apply<F: Fn(u8) -> W<u8>>(f: F) -> W<u8> {
    f(1)
}
fn wrap_all<T: Copy>(xs: &[T]) -> Vec<P<T>> {
    xs.iter().map(|&x| (x, x)).map(|(a, b)| P(a, b)).collect()
}
fn ctor_generic<T>(x: T) -> W<T> {
    let f = W;
    f(x)
}
fn main() {
    let v: Vec<W<u8>> = vec![1u8, 2].into_iter().map(W).collect();
    let e: Vec<E<i16>> = vec![3i16].into_iter().map(E::A).collect();
    let f = E::B;
    let w = W::<u16>;
    println!("{:?} {:?} {:?} {:?}", v, e, f(4u32, 5), w(6));
    println!("{:?} {:?} {:?}", apply(W), wrap_all(&[7u8]), ctor_generic('c'));
    let g = if v.len() > 1 { W } else { W };
    println!("{:?} {}", g(8i8), std::mem::size_of_val(&g));
}
