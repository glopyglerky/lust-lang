// A generic `impl Trait` fn whose hidden type holds a closure, called
// inside a closure body: the outer closure's return type names the
// instance's closure once mono has made it (a build failure before).
fn mk<I: IntoIterator>(i: I) -> impl Iterator<Item = I::Item> {
    i.into_iter().map(|x| x)
}
fn main() {
    let v = vec![1u32, 2, 3, 4];
    let o = None::<u8>.map(|_| mk(&v));
    let n = Some(1u8).map(|_| mk(&v).count());
    println!("{} {:?}", o.is_none(), n);
}
