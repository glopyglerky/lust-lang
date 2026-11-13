// `VecDeque::retain` drops the removed elements as std does: kept ones
// swap to the front, then one truncate drops the rest in their order
// after the swaps (not each one at its removal).
use std::collections::VecDeque;
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}
fn main() {
    let mut q: VecDeque<D> = VecDeque::new();
    for x in [1, 2, 0, 1, 6] {
        q.push_back(D(x));
    }
    q.push_front(D(8));
    q.push_front(D(6));
    q.retain(|x| x.0 > 3);
    println!("{}", q.len());
    let mut n = 0;
    q.retain(|_| {
        n += 1;
        n != 1
    });
    println!("{} {}", q.len(), n);
}
