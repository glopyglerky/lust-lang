// std collection iterator structs (VecDeque, BTreeMap, BTreeSet iterators) behind `&mut`: as iterator
// values and IntoIterator arguments, plain and through generic instances; hints and capacity.
fn take2<I: IntoIterator>(i: I) -> Vec<I::Item> { i.into_iter().take(2).collect() }
fn ext<I: IntoIterator>(i: I) -> Vec<I::Item> { let mut v = Vec::new(); v.extend(i); v }
fn main_a() {
    let c: std::collections::VecDeque<u32> = (1..7).collect(); let mut a = c.into_iter();
    let r = &mut a;
    let s: Vec<u32> = take2(r);
    println!("{:?}", s);
    println!("{:?}", a.next());
}
fn main_b() {
    let c: std::collections::BTreeMap<u8, u8> = (1..7).map(|x| (x, x * 2)).collect(); let mut a = c.iter();
    
    let s: Vec<(&u8, &u8)> = a.by_ref().skip(1).collect(); println!("{} {}", s.len(), s.capacity());
    println!("{:?}", s);
    println!("{:?}", a.next());
}
fn main_c() {
    let c: std::collections::VecDeque<u32> = (1..7).collect(); let mut a = c.iter();
    
    let s = ((&mut a).size_hint(), (&mut a).skip(1).size_hint());
    println!("{:?}", s);
    println!("{:?}", a.next());
}
fn main() {
    main_a();
    main_b();
    main_c();
}
