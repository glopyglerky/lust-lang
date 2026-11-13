// std::alloc::Layout and LayoutError (ts-rust leak_arena.rs).
use std::alloc::{Layout, LayoutError};
#[allow(dead_code)]
struct P { a: u8, b: u64, c: u16 }
fn show(r: Result<Layout, LayoutError>) -> String {
    match r {
        Ok(l) => format!("ok {} {}", l.size(), l.align()),
        Err(e) => format!("err {e} {e:?}"),
    }
}
fn main() {
    let l = Layout::new::<u64>();
    println!("{:?} {} {}", l, l.size(), l.align());
    println!("{:#?}", Layout::array::<u16>(3).unwrap());
    println!("{:?}", Layout::new::<P>());
    println!("{:?} {:?}", Layout::new::<()>(), Layout::new::<[u32; 0]>());
    println!("{:?}", Layout::for_value(&[1u32, 2, 3][..]));
    println!("{:?}", Layout::for_value("hello"));
    for (s, a) in [(7, 3), (7, 0), (8, 8), (usize::MAX, 1), (isize::MAX as usize, 1), (isize::MAX as usize, 2), (isize::MAX as usize - 1, 2)] {
        println!("{s} {a}: {}", show(Layout::from_size_align(s, a)));
    }
    for n in [0, 1, usize::MAX / 8, usize::MAX / 16, isize::MAX as usize / 8 + 1] {
        println!("{n}: {}", show(Layout::array::<u64>(n)));
    }
    println!("{}", show(Layout::array::<()>(usize::MAX)));
    let a = Layout::new::<u32>();
    let b = a;
    println!("{} {}", a == b, a == Layout::new::<i32>());
    println!("{}", Layout::array::<u8>(4).expect("size") == Layout::new::<[u8; 4]>());
}
