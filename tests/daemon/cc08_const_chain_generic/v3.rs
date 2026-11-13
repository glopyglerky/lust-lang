const fn cf() -> usize { 6 }
const N: usize = cf();
const M: usize = N * 2;
fn get<const K: usize>() -> usize {
    K
}
struct W<const K: usize>;
impl<const K: usize> W<K> {
    fn v(&self) -> usize {
        K + 1
    }
}
fn pair<T: Copy>(x: T) -> (T, T) {
    (x, x)
}
fn main() {
    println!("{} {} {:?}", get::<M>(), W::<N>.v(), pair(M));
}
