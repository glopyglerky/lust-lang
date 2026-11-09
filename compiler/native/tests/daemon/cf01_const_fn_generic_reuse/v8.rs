const fn scale(x: u32) -> u32 {
    x * 4
}
const N: u32 = scale(3);
static TABLE: [u32; 3] = [scale(1), scale(2), N];
const fn twice(x: u64) -> u64 {
    x * 2
}
const M: u64 = twice(21);

fn plain_extra() -> u32 {
    7
}

fn plain_more() -> u32 {
    8
}

struct Wrap<T> {
    v: T,
}

impl<T: Copy + std::fmt::Debug> Wrap<T> {
    fn show(&self) -> String {
        format!("{:?}", self.v)
    }
}

fn total<T: Into<u64> + Copy>(xs: &[T]) -> u64 {
    xs.iter().map(|&x| x.into()).sum::<u64>() + N as u64
}

fn arr() -> [u8; scale(2) as usize] {
    [1; scale(2) as usize]
}

fn sized<const K: usize>() -> usize {
    K * 10
}

fn main() {
    println!("{} {}", plain_extra(), plain_more());
    let w = Wrap { v: N };
    println!("{} {} {:?} {}", w.show(), total(&[1u32, 2, 3]), TABLE, arr().len());
    println!("{}", sized::<{ scale(1) as usize }>());
    println!("{}", M + total(&[4u16]));
}
