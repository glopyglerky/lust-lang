// Promoted references through one body path: an iterator builtin (lower/iter.rs expands it in a copy of the body). Each producer
// returns a promoted reference (scalar, aggregate, nested, const ref,
// zero-sized) from a plain fn, a closure or a generic fn; main clobbers the
// stack through other calls and then reads every reference. A promoted temp
// in a frame slot (a lost Body::promoted list) prints garbage here
// (SOL-COVERAGE-4 D1).
#![allow(dead_code)]
#[derive(Debug)]
struct P {
    a: u8,
    b: u16,
}
#[derive(Debug)]
struct Unit;
const R: &P = &P { a: 4, b: 5 };
thread_local! { static K: u32 = 3; }

#[inline(never)]
fn clobber(seed: u8) -> u64 {
    let mut a = [seed; 512];
    let mut i = 0;
    while i < 512 {
        a[i] = (i * 7) as u8 ^ seed;
        i += 1;
    }
    let mut s = 0u64;
    let mut j = 0;
    while j < 512 {
        s += a[j] as u64;
        j += 1;
    }
    s
}

fn plain_scalar(v: Vec<u32>) -> (u32, &'static u8) {
    (v.iter().sum::<u32>(), &7)
}

fn plain_aggregate(v: Vec<u32>) -> (u32, &'static (u8, char, [u16; 2])) {
    (v.into_iter().next().unwrap_or(0), &(1, 'z', [2, 3]))
}

fn plain_nested(v: Vec<u32>) -> (u32, &'static &'static u8) {
    ({ let mut s = 0; for x in v.iter() { s += x; } s }, &&8)
}

fn plain_constref(v: Vec<u32>) -> (u32, &'static P) {
    (v.iter().sum::<u32>(), R)
}

fn plain_zst_slice(v: Vec<u32>) -> (u32, &'static [u32]) {
    (v.into_iter().next().unwrap_or(0), &[])
}

fn closure_nested(v: Vec<u32>) -> (u32, &'static &'static u8) {
    let f = move || -> (u32, &'static &'static u8) { ({ let mut s = 0; for x in v.iter() { s += x; } s }, &&8) };
    f()
}

fn generic_zst_slice<X: Copy>(x: X, v: Vec<u32>) -> (X, u32, &'static [u32]) {
    let (n, r) = (v.iter().sum::<u32>(), &[]);
    (x, n, r)
}

fn closure_constref(v: Vec<u32>) -> (u32, &'static P) {
    let f = move || -> (u32, &'static P) { (v.into_iter().next().unwrap_or(0), R) };
    f()
}

fn generic_scalar<X: Copy>(x: X, v: Vec<u32>) -> (X, u32, &'static u8) {
    let (n, r) = ({ let mut s = 0; for x in v.iter() { s += x; } s }, &7);
    (x, n, r)
}

fn main() {
    let r0 = plain_scalar(vec![0, 1]);
    let r1 = plain_aggregate(vec![1, 1]);
    let r2 = plain_nested(vec![2, 1]);
    let r3 = plain_constref(vec![3, 1]);
    let r4 = plain_zst_slice(vec![4, 1]);
    let r5 = closure_nested(vec![5, 2]);
    let r6 = generic_zst_slice(6u16, vec![6, 3]);
    let r7 = closure_constref(vec![7, 2]);
    let r8 = generic_scalar(8u16, vec![8, 3]);
    let k = clobber(0xa5) + clobber(0x3c);
    println!("0: {:?}", r0);
    println!("1: {:?}", r1);
    println!("2: {:?}", r2);
    println!("3: {:?}", r3);
    println!("4: {:?}", r4);
    println!("5: {:?}", r5);
    println!("6: {:?}", r6);
    println!("7: {:?}", r7);
    println!("8: {:?}", r8);
    println!("{k}");
}
