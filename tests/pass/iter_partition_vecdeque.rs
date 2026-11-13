// `partition` into `VecDeque`s (ts-rust's ipc conn_async), from a deque's
// own `into_iter` and from a user iterator: core's `Default` then
// `extend_one` (`push_back`), so capacity and the ring head are a Vec's.
use std::collections::VecDeque;

struct Count(u8);

impl Iterator for Count {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 < 7 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
}

fn main() {
    let mut msgs: VecDeque<(String, u32)> = VecDeque::new();
    for i in 0..9u32 {
        msgs.push_back((format!("m{i}"), i));
    }
    let (resp, rest): (VecDeque<_>, VecDeque<_>) = std::mem::take(&mut msgs).into_iter().partition(|(_, n)| n % 3 == 0);
    println!("{:?} {}", resp, resp.capacity());
    println!("{:?} {} {}", rest, rest.capacity(), msgs.len());
    let mut rest = rest;
    rest.push_front(("f".to_string(), 99));
    println!("{:?}", rest.as_slices());
    let (a, b): (Vec<u8>, Vec<u8>) = Count(0).partition(|x| x % 2 == 0);
    println!("{:?} {:?} {} {}", a, b, a.capacity(), b.capacity());
    let (c, d): (VecDeque<u8>, VecDeque<u8>) = Count(3).partition(|x| *x > 5);
    println!("{:?} {:?}", c, d);
}
