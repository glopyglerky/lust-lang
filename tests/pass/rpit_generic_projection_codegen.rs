// A generic `impl Trait` fn whose hidden type is a projection of its
// params (`I::IntoIter`): each instance's return type normalizes at its
// args, by reference and by value.
fn items<I: IntoIterator>(i: I) -> impl Iterator<Item = I::Item> {
    i.into_iter()
}
fn doubled<I: IntoIterator<Item = u32>>(i: I) -> impl Iterator<Item = u32> {
    i.into_iter().map(|x| x * 2)
}
fn main() {
    let v = vec![1u32, 2, 3];
    println!("{}", items(&v).count());
    let s: u32 = doubled(v.clone()).sum();
    println!("{}", s);
    let mut n = 0;
    for x in items(v) {
        n += x;
    }
    println!("{}", n);
}
