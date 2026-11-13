use std::collections::VecDeque;
fn main() {
    let mut q: VecDeque<String> = VecDeque::new();
    q.push_back(String::from("y"));
    println!("{:?} {} {}", q, q.len(), q.capacity());
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    q.push_back(String::from("y"));
    q.push_back(String::from("z"));
    q.push_back(String::from("z"));
    q.push_back(String::from("z"));
    if q.len() > 2 { q.insert(1, String::from("x")); } println!("{:?}", q.remove(2));
    q.push_back(String::from("x"));
    q.push_front(String::from("y"));
    println!("{:?} {:?}", q.pop_front(), q.pop_back());
    q.push_front(String::from("z"));
    q.truncate(4); q.shrink_to_fit(); println!("{:?} {}", q, q.capacity());
    println!("{:?} {:?}", q.pop_front(), q.pop_back());
}
