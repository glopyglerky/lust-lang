// `array::IntoIter::as_slice`, `as_mut_slice`, `len`, and
// `size_hint` over the remaining range (fx-array-iter).
fn main() {
    let mut it = [String::from("a"), String::from("bb"), String::from("ccc"), String::from("dddd")].into_iter();
    println!("{} {:?} {:?}", it.len(), it.size_hint(), it.as_slice());
    let x = it.next().unwrap();
    println!("{} {} {:?}", x, it.len(), it.as_slice());
    let y = it.next_back().unwrap();
    println!("{} {} {:?}", y, it.len(), it.as_slice());
    it.as_mut_slice()[0].push('!');
    println!("{:?}", it.as_slice());
    let rest: Vec<String> = it.collect();
    println!("{:?}", rest);
    // Empty and exhausted views.
    let mut e: std::array::IntoIter<String, 0> = [].into_iter();
    println!("{} {:?}", e.len(), e.as_slice());
    println!("{:?}", e.next());
    let mut one = [vec![1u32]].into_iter();
    let _ = one.next();
    println!("{} {:?}", one.len(), one.as_slice());
    // `as_slice` on a Copy iterator too.
    let mut c = [1, 2, 3].into_iter();
    let _ = c.next();
    println!("{:?} {}", c.as_slice(), c.len());
    c.as_mut_slice()[0] = 20;
    println!("{:?}", c.collect::<Vec<_>>());
}
