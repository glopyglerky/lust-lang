// feature: iterators.chains
// form: map/filter/collect chain, sum, and for over a method call, as in ts-rust list code
fn main() {
    let v = vec![1, 2, 3, 4];
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).filter(|x| *x > 4).collect();
    let total: i32 = v.iter().copied().skip(1).take(2).sum();
    let mut letters = 0;
    for _c in "abcd".chars() { letters += 1; }
    println!("{:?} {} {}", doubled, total, letters);
}
