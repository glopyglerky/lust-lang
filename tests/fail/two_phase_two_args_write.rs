// expect: L0303
// rustc E0499: `a`'s two-phase reservation is a shared borrow while the
// later arguments run, so `a.push(9)` there cannot borrow it mutably.
fn g(a: &mut Vec<u32>, b: &mut Vec<u32>, n: usize) -> usize { a.len() + b.len() + n }
fn main() {
    let mut x = vec![1u32];
    let mut y = vec![2u32];
    let a = &mut x;
    let b = &mut y;
    println!("{}", g(a, b, { a.push(9); 0 }));
}
