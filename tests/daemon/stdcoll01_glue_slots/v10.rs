use std::collections::VecDeque;
fn main() {
    let mut q: VecDeque<u8> = VecDeque::with_capacity(0);
    q.push_front(3);
    if q.len() > 0 { q.insert(1, 8); } println!("{:?}", q.remove(2));
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    q.push_front(5);
    println!("{:?} {:?} {:?}", q.get(4), q.front(), q.back());
    println!("{:?} {:?} {:?}", q.get(4), q.front(), q.back());
    println!("{:?} {} {}", q, q.len(), q.capacity());
    q.push_back(7);
    println!("{:?} {:?}", q.pop_front(), q.pop_back());
    println!("{:?} {} {}", q, q.len(), q.capacity());
    q.push_back(8);
    println!("{:?} {:?}", q.pop_front(), q.pop_back());
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
}
