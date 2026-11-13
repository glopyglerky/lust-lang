// `std::array::from_fn` in its path, import, and turbofish forms
// (W-BENCH3 bug 13), with closures, fn items, and captures.
use std::array::from_fn;
fn sq(i: usize) -> usize {
    i * i
}
fn main() {
    let p: Box<[Option<u32>; 4]> = Box::new(std::array::from_fn(|_| None));
    println!("{:?}", p);
    let a: [String; 3] = from_fn(|i| i.to_string());
    println!("{:?}", a);
    let b = core::array::from_fn::<u8, 3, _>(|i| i as u8 + 1);
    println!("{:?}", b);
    let c: [usize; 4] = std::array::from_fn(sq);
    println!("{:?}", c);
    let mut n = 10;
    let d: [i32; 3] = std::array::from_fn(|_| {
        n += 1;
        n
    });
    println!("{:?} {}", d, n);
    let e: [u8; 0] = std::array::from_fn(|_| unreachable!());
    println!("{}", e.len());
}
