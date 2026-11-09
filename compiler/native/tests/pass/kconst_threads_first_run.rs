// Threads that run constant sites for the first time together see one address per site, the
// one the site's first run published: promoted arrays and enums, str and byte literals. Each
// comparison is between runs of the same fn (between fns rustc's answer depends on its
// codegen unit split).
use std::sync::{Arc, Barrier};
use std::thread;

macro_rules! sites {
    ($($name:ident = $v:expr;)*) => {
        $(
            fn $name() -> ([usize; 4], u64) {
                let a: &'static [u64; 3] = &[$v, $v + 1, $v + 2];
                let s: &'static str = concat!("site ", stringify!($name));
                let o: &'static Option<(u8, u32)> = &Some((1, $v as u32));
                let b: &'static [u8] = b"bytes";
                let sum = a.iter().sum::<u64>() + s.len() as u64 + o.map_or(0, |(x, y)| x as u64 + y as u64) + b.len() as u64;
                ([a.as_ptr() as usize, s.as_ptr() as usize, o as *const _ as usize, b.as_ptr() as usize], sum)
            }
        )*
        fn all() -> Vec<fn() -> ([usize; 4], u64)> {
            vec![$($name),*]
        }
    };
}

sites! {
    s0 = 10; s1 = 11; s2 = 12; s3 = 13; s4 = 14; s5 = 15; s6 = 16; s7 = 17;
    s8 = 18; s9 = 19; s10 = 20; s11 = 21; s12 = 22; s13 = 23; s14 = 24; s15 = 25;
}

fn main() {
    let n = 8;
    let barrier = Arc::new(Barrier::new(n));
    let handles: Vec<_> = (0..n)
        .map(|_| {
            let b = Arc::clone(&barrier);
            thread::spawn(move || {
                b.wait();
                all().iter().map(|f| f()).collect::<Vec<_>>()
            })
        })
        .collect();
    let runs: Vec<Vec<([usize; 4], u64)>> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let again: Vec<([usize; 4], u64)> = all().iter().map(|f| f()).collect();
    let same = runs.iter().all(|r| *r == again);
    println!("{n} threads, {} sites: one address per site: {same}", again.len());
    let sums: Vec<u64> = again.iter().map(|(_, s)| *s).collect();
    println!("sums {sums:?}");
}
