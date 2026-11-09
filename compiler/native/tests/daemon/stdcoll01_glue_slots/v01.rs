use std::collections::VecDeque;
fn main() {
    let mut q: VecDeque<i64> = VecDeque::new();
    q.push_back(-90);
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    q.push_back(21);
    println!("{:?} {:?} {:?}", q.get(4), q.front(), q.back());
    q.push_back(-36);
    q.push_front(-30);
    println!("{:?} {:?}", q.pop_front(), q.pop_back());
    if q.len() > 2 { q.insert(1, 68); } println!("{:?}", q.remove(3));
    q.push_back(26);
    q.push_front(63);
    q.push_front(16);
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    q.push_front(36);
    q.push_front(1);
    println!("{:#?} {} {}", q, q.len(), q.capacity());
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
}
