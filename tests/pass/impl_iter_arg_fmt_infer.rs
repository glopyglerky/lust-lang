// An item binding (`I: Iterator<Item = (T, T)>`) fixes `T` at the call
// only when the obligation queue runs, so `{:?}` of the `Vec<T>` result
// checks its Debug need after that.
fn firsts<T: Clone, I: Iterator<Item = (T, T)>>(it: I) -> Vec<T> { let mut v = Vec::new(); for (a, _) in it { v.push(a); } v }
fn evens(it: impl Iterator<Item = u32>) -> Vec<u32> { it.filter(|x| x % 2 == 0).collect() }
fn main() {
    println!("{:?}", firsts(vec![(1, 2), (3, 4)].into_iter()));
    let r = firsts(vec![("a", "b")].into_iter());
    println!("{:?} {:?}", r, evens(0..7));
}
