// expect: L0110
// rustc E0308: `A::<u32>` takes the default `V = u8`, so it is not D<u32, u16>.
#[derive(Default)]
struct D<K, V> {
    k: Vec<K>,
    v: Vec<V>,
}
type A<K, V = u8> = D<K, V>;
fn main() {
    let a = A::<u32>::default();
    let b: D<u32, u16> = a;
    println!("{}", b.k.len() + b.v.len());
}
