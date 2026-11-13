// Builtin iterators satisfy `Iterator` and `IntoIterator` bounds:
// `I::Item`, `next`, and `for` in generic bodies (W-STD-A phase 3).
fn first<I: Iterator>(mut it: I) -> Option<I::Item> {
    it.next()
}
fn total<I: Iterator<Item = u32>>(it: I) -> u32 {
    let mut s = 0;
    for x in it {
        s += x;
    }
    s
}
fn len_of<I: IntoIterator>(it: I) -> usize {
    let mut n = 0;
    for _ in it {
        n += 1;
    }
    n
}
fn main() {
    let v = vec![3u32, 4, 5];
    println!("{:?}", first(v.iter().copied()));
    println!("{}", total(v.iter().copied()));
    println!("{}", total(v.iter().map(|x| x * 2)));
    println!("{}", len_of(v.iter().filter(|x| **x > 3)));
    println!("{}", total((1..4).map(|x| x as u32)));
    println!("{:?}", first(v.into_iter().rev()));
}
