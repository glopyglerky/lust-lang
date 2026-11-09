// `std::collections::VecDeque` is lust std source (W-STD-C): push and pop
// at both ends, get, insert, remove, retain, extend through append,
// capacity growth by the RawVec rule, and Debug.
use std::collections::VecDeque;

fn main() {
    let mut q = VecDeque::new();
    println!("{} {}", q.len(), q.capacity());
    q.push_back(2);
    q.push_front(1);
    q.push_back(3);
    println!("{:?} {} {:?} {:?}", q, q.capacity(), q.front(), q.back());
    println!("{:?} {:?} {:?}", q.pop_front(), q.pop_back(), q);
    for i in 0..10 {
        if i % 2 == 0 {
            q.push_back(i);
        } else {
            q.push_front(i);
        }
    }
    println!("{:?} {} {}", q, q.len(), q.capacity());
    q.insert(3, 100);
    println!("{:?} {:?} {:?}", q.remove(0), q.get(2), q.get(99));
    q.retain(|x| x % 3 != 0);
    println!("{:?} {}", q, q.contains(&100));
    if let Some(x) = q.front_mut() {
        *x += 1000;
    }
    q.swap(0, 1);
    println!("{:?}", q);
    let sum: i32 = q.iter().sum();
    let big: Vec<&i32> = q.iter().filter(|x| **x > 5).collect();
    println!("{} {:?}", sum, big);
    for x in q.iter_mut() {
        *x *= 2;
    }
    let mut other: VecDeque<i32> = VecDeque::new();
    other.push_back(7);
    other.push_back(8);
    q.append(&mut other);
    println!("{:?} {:?} {}", q, other, other.is_empty());
    let tail = q.split_off(4);
    println!("{:?} {:?}", q, tail);
    q.truncate(2);
    let w: VecDeque<String> = VecDeque::with_capacity(5);
    println!("{:?} {} {}", q, w.capacity(), w.len());
    let mut c = VecDeque::with_capacity(2);
    c.push_back(String::from("a"));
    c.push_back(String::from("b"));
    c.push_front(String::from("z"));
    println!("{:?} {} {}", c, c.capacity(), c == c.clone());
    c.clear();
    c.shrink_to_fit();
    println!("{} {}", c.len(), c.capacity());
    let v: Vec<String> = c.into_iter().collect();
    println!("{:?}", v);
}
