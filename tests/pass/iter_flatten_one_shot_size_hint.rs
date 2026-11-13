// std's OneShot flatten: inner iterators of at most one item (Option,
// Result, Once, OnceWith, array::IntoIter<T, 0 or 1>) give the hint
// (0, the outer's upper).
fn main() {
    let v = vec![Some(1), None, Some(3)];
    let mut it = v.clone().into_iter().flatten();
    println!("{:?}", it.size_hint());
    it.next();
    println!("{:?}", it.size_hint());
    println!("{:?}", v.iter().flatten().size_hint());
    println!("{:?}", [[1], [2]].into_iter().flatten().size_hint());
    println!("{:?}", vec![Ok::<i32, ()>(1)].into_iter().flatten().size_hint());
    let mut it5 = (0..4).flat_map(|x| std::iter::once(x)).filter(|x| x % 2 == 0);
    println!("{:?}", it5.size_hint());
    it5.next();
    println!("{:?}", it5.size_hint());
    println!("{:?}", (0..4).flat_map(|x| std::iter::once_with(move || x).map(|y| y + 1)).size_hint());
    println!("{:?}", (0..4).flat_map(|x| [x; 0]).size_hint());
    println!("{:?}", (0..3).flat_map(|x| [x].into_iter()).size_hint());
    let w: Vec<i32> = (0..3).flat_map(|x| std::iter::once(x)).collect();
    println!("{:?} {}", w, w.capacity());
}
