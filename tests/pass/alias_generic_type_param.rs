// A generic alias `type Pair<T> = (T, T)` resolves at each use.
type Pair<T> = (T, T);
fn swap<T>(p: Pair<T>) -> Pair<T> { (p.1, p.0) }
fn main() {
    let p: Pair<i32> = (1, 2);
    let q = swap(p);
    println!("{} {}", q.0, q.1);
}
