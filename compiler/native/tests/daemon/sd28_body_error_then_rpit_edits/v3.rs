// A const fn body error stays while a fn that returns impl Trait changes its hidden type (v3) and
// back (v4): the warm daemon reuses the program it kept before the error, and the bodies that the
// error answers checked again against their own program are read-set candidates (patchseq 7, 1408).
mod consts {
pub const fn len() -> usize {
    // @c1
    4
}

pub const fn seed() -> i64 {
    // @c2
    3
}

pub const fn unread(x: i64) -> i64 {
    // @c4
    x + 1
}

pub const fn width() -> usize {
    2
}

pub const fn param() -> usize {
    // @c5
    let _s90947 = String::from("90947");
    1
}

pub const LEN: usize = len();
pub static TABLE: [i64; 2] = [seed(), seed() + 1];

fn pick<const N: usize>() -> usize {
    N
}

pub fn run() -> i64 {
    let a: [u8; 4] = [0; LEN];
    let _w: [u8; 2] = [0u8; width()];
    // @c3
    a.len() as i64 + TABLE[1] + unread(2) + pick::<{ param() }>() as i64
}
}
mod gated {
pub fn helper(x: i64) -> i64 {
    // @g1
    x * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        // @g2
        assert_eq!(helper(1), 2);
    }
}

#[cfg(not(unix))]
pub fn os_name() -> &'static str {
    // @g3
    "other"
}

pub struct S;

impl S {
    #[cfg(windows)]
    fn only_windows(&self) -> i64 {
        // @g4
        1
    }

    pub fn get(&self) -> i64 {
        // @g5
        3
    }
}

pub fn run() -> i64 {
    // @g6
    helper(3) + S.get()
}
}
mod items {
use std::sync::{LazyLock, OnceLock};

pub fn mode() -> u32 {
    static MODE: OnceLock<u32> = OnceLock::new();
    // @i1
    *MODE.get_or_init(|| 7)
}

pub fn table() -> u32 {
    static TABLE: LazyLock<Vec<u32>> = LazyLock::new(|| vec![1, 2, 3]);
    // @i2
    TABLE.len() as u32
}

pub fn nested() -> i64 {
    const K: i64 = 4;
    fn inner(x: i64) -> i64 {
        // @i3
        x + K
    }
    // @i4
    inner(2)
}

pub fn uses() -> i64 {
    use std::collections::HashMap;
    let mut m: HashMap<i64, i64> = HashMap::new();
    // @i5
    m.insert(1, 2);
    m.len() as i64
}

pub fn run() -> i64 {
    mode() as i64 + table() as i64 + nested() + uses()
}
}
mod macs {
use std::fmt::Write;

macro_rules! dbl {
    ($e:expr) => {
        $e * 2
    };
}

#[derive(Clone, Copy)]
pub enum K {
    A,
    B(i64),
}

pub fn is_b(k: K) -> bool {
    // @m1
    matches!(k, K::B(_))
}

pub fn show(k: K) -> String {
    let mut s = String::new();
    // @m2
    write!(s, "{}", is_b(k) as i64).unwrap();
    s
}

pub fn run() -> i64 {
    debug_assert!(is_b(K::B(1)));
    // @m3
    show(K::A).len() as i64 + is_b(K::B(2)) as i64 + dbl!(1)
}
}
mod rpit {
pub struct Table {
    pub r16: &'static [(u16, u16, u16)],
    pub r32: &'static [(u32, u32, u32)],
}

impl Table {
    pub fn starts(&self) -> impl Iterator<Item = i64> + '_ {
        // @p1
        self.r16.iter().map(|t| t.0 as i64)
    }
}

pub struct Bag<T> {
    pub items: Vec<T>,
}

impl<T> Bag<T> {
    pub fn all(&self) -> impl Iterator<Item = &T> + '_ {
        // @p6
        self.items.iter()
    }
}

pub fn ranges(x: &Table) -> impl Iterator<Item = (i64, i64, i64)> + '_ {
    // @p2
    let a = x.r16.iter().map(|&(lo, hi, s)| (i64::from(lo), i64::from(hi), i64::from(s)));
    let b = x.r32.iter().map(|&(lo, hi, s)| (lo as i64, hi as i64, s as i64));
    a.chain(b)
}

pub fn adder(n: i64) -> impl Fn(i64) -> i64 {
    // @p3
    move |x| x + n
}

pub fn token() -> impl Sized {
    // @p4
    7u16
}

fn need_send<T: Send>(_t: T) {}

pub static T: Table = Table { r16: &[(1, 5, 1), (10, 20, 2)], r32: &[(0x10000, 0x10010, 1)] };

pub fn run() -> i64 {
    let mut n = 0;
    for (lo, hi, s) in ranges(&T) {
        n += (hi - lo) / s + 1;
    }
    need_send(adder(1));
    need_send(token());
    let bag = Bag { items: vec![4i64, 5] };
    // @p5
    n + adder(2)(3) + T.starts().sum::<i64>() + bag.all().next().copied().unwrap_or(0)
}
}
mod sigs {
pub const N: usize = 2;

pub fn skip(_: &str) -> i64 {
    // @s1
    1
}

pub fn pair((a, b): (i64, i64), [c, _]: [i64; N]) -> i64 {
    // @s2
    a + b + c
}

pub fn first(xs: [u8; 4]) -> [u8; N] {
    // @s3
    [xs[0], xs[1]]
}

pub struct S;

impl S {
    pub fn get(&self, (x, _y): (i64, i64), _: u8) -> i64 {
        // @s4
        x
    }
}

pub fn outer(k: i64) -> i64 {
    fn twice(v: i64) -> i64 {
        // @s5
        v * 2
    }
    // @s6
    twice(k) + 1
}

pub fn run() -> i64 {
    skip("a") + pair((1, 2), [3, 4]) + first([1, 2, 3, 4])[1] as i64 + S.get((5, 6), 7) + outer(3)
}
}
mod tmpl {
use std::any::Any;

pub trait AnyValue: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        // @t1
        self
    }
}

pub trait Name {
    fn name(&self) -> String;
}

impl Name for i32 {
    fn name(&self) -> String {
        // @t2
        format!("i32 {self}")
    }
}

fn show(x: &dyn Name) -> String {
    x.name()
}

fn wrap<T: Name + Clone>(x: T) -> Vec<T> {
    // @t3
    let v = vec![x.clone(), x];
    v
}

// A template whose body holds a closure (W-PATCH-MISS2).
fn twice<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    // @t4
    #[allow(unused)] let _a2550 = 1;
    let g = |y: i64| f(f(y));
    g(x)
}

pub fn run() -> i64 {
    let n = if 5u8.as_any().is::<u8>() { 1 } else { 0 };
    n + wrap(3i32).len() as i64 + show(&1i32).len() as i64 + twice(|v| v + 1, 1)
}
}

// The root carries the std splice (`and_then` is a std source method):
// an edit here moves it (W-PATCH-MISS2).
fn pick(o: Option<i64>) -> i64 {
    // @r1
    let _v11612: u32 = 11612;
    o.and_then(|n| n.checked_add(1)).unwrap_or(0)
}

fn main() {
    let t = items::run() + gated::run() + consts::run() + tmpl::run() + macs::run() + sigs::run() + rpit::run();
    // @r2
    println!("{}", t + pick(Some(2)));
}
