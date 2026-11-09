// std::hint::black_box returns its argument: by path, imported, through a
// glob, with a turbofish, on borrows, on owned values (moved in and out,
// dropped once), and on a mutable borrow written through.
use std::hint::black_box;

mod g {
    use std::hint::*;
    pub fn twice(x: u32) -> u32 {
        black_box(x) * 2
    }
}

struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}

fn main() {
    let v = vec![1, 2, 3];
    let s: i32 = black_box(&v).iter().sum();
    let w = black_box(v);
    println!("{} {:?}", s, w);
    println!("{}", std::hint::black_box(5u8) + core::hint::black_box(2));
    let mut x = 3;
    *black_box(&mut x) += 1;
    println!("{x}");
    let st = black_box(String::from("hi"));
    println!("{st} {}", g::twice(21));
    let d = black_box(D(1));
    println!("{}", black_box::<u64>(7));
    let _ = black_box(D(2));
    println!("{}", d.0);
}
