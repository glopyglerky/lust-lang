// OnceWith is a DoubleEndedIterator (next_back is next).
fn main() {
    let v: Vec<u8> = std::iter::once_with(|| 3).rev().collect();
    println!("{:?} {}", v, v.capacity());
    let mut it = std::iter::once_with(|| String::from("a")).chain(std::iter::once(String::from("b"))).rev();
    println!("{:?} {:?} {:?}", it.next(), it.size_hint(), it.next());
    let w: Vec<i64> = std::iter::once(1).chain(std::iter::once_with(|| 2)).map(|x| x * 3).rev().collect();
    println!("{:?} {}", w, w.capacity());
    let mut o = std::iter::once_with(|| 5u32);
    println!("{:?} {:?}", o.next_back(), o.next_back());
}
