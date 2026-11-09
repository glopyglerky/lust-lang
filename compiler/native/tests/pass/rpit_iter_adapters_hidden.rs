// Adapters and consumers over `impl Iterator` values (W-STD-B slice 8):
// items, size_hint, and collect capacities are the hidden type's, as rustc.
fn evens() -> impl Iterator<Item = u32> {
    vec![2u32, 4].into_iter()
}

struct Up(u32);

impl Iterator for Up {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 4 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
}

fn ups() -> impl Iterator<Item = u32> {
    Up(0)
}

fn main() {
    let s: u32 = evens().sum();
    println!("{}", s);
    println!("{:?}", evens().map(|x| x + 1).collect::<Vec<_>>());
    println!("{}", evens().count());
    for x in evens() {
        print!("{} ", x);
    }
    println!();
    let mut it = evens();
    println!("{:?} {:?} {:?}", it.next(), it.size_hint(), it.next());
    println!("{:?}", ups().zip(evens()).collect::<Vec<_>>());
    println!("{:?} {:?}", ups().max(), ups().position(|x| x == 3));
    let v: Vec<u32> = ups().chain(evens()).filter(|x| x % 2 == 0).collect();
    println!("{:?} {}", v, v.capacity());
    println!("{:?}", evens().size_hint());
}
