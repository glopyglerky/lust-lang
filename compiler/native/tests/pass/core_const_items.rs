use std::mem::{align_of, size_of};

const BASE: i64 = 40;
const TWO: i64 = BASE + 2;
const PAIR: (i64, i64) = (1, TWO);
const TABLE: [i64; 3] = [10, 20, 30];
const WORD: &str = "hi";
const FLAG: bool = TWO > 10;

const _: () = assert!(BASE < 100);
const _: () = assert!(size_of::<u64>() == 8);

const fn double(x: i64) -> i64 {
    x * 2
}

const fn fib(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    let mut i = 0;
    while i < n {
        let t = a + b;
        a = b;
        b = t;
        i += 1;
    }
    a
}

const BIG: u64 = fib(10);

struct Point {
    x: i64,
    y: i64,
}

const ORIGIN: Point = Point { x: 0, y: 0 };

static GREETING: &str = "static";
static NUMS: [i64; 2] = [7, 8];
static TOP: Point = Point { x: 3, y: 4 };

fn get() -> i64 {
    NUMS[0]
}

fn main() {
    println!("{} {} {}", BASE, TWO, double(21));
    println!("{} {}", PAIR.0, PAIR.1);
    println!("{}", TABLE[2]);
    println!("{} {}", WORD, FLAG);
    println!("{}", BIG);
    println!("{} {}", ORIGIN.x, ORIGIN.y);
    println!("{}", GREETING);
    println!("{} {}", NUMS[1], TOP.y);
    println!("{}", get());
    println!("{} {}", size_of::<u32>(), align_of::<u64>());
    let x = const { 1 + 2 };
    println!("{}", x);
    const { assert!(size_of::<(u8, u64)>() == 16) };
    match TWO {
        42 => println!("answer"),
        _ => println!("no"),
    }
    match 42 {
        TWO => println!("named"),
        _ => println!("other"),
    }
}
