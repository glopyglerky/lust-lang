// A trait const typed by the trait's own associated type
// (`const MIN: Self::Primitive`), written as `Self::Primitive` in the
// impl too, read concretely, generically, in const items, in casts, and
// through `Debug` with a projection-bound impl parameter (jiff-core's
// bounds.rs shape).
use std::fmt;

pub trait Primitive: Copy + fmt::Debug + fmt::Display + PartialOrd {
    fn as_i64(self) -> i64;
    fn from_i64(n: i64) -> Self;
}
impl Primitive for i8 {
    fn as_i64(self) -> i64 {
        self as i64
    }
    fn from_i64(n: i64) -> i8 {
        n as i8
    }
}
impl Primitive for i64 {
    fn as_i64(self) -> i64 {
        self
    }
    fn from_i64(n: i64) -> i64 {
        n
    }
}

pub trait Bounds: Sized {
    const WHAT: &'static str;
    const MIN: Self::Primitive;
    const MAX: Self::Primitive;
    type Primitive: Primitive;

    fn check(n: i64) -> Option<Self::Primitive> {
        if !(Self::MIN.as_i64() <= n && n <= Self::MAX.as_i64()) {
            return None;
        }
        Some(<Self::Primitive as Primitive>::from_i64(n))
    }
}

#[derive(Clone, Copy)]
struct Hour(());
impl Bounds for Hour {
    const WHAT: &'static str = "hour";
    const MIN: Self::Primitive = 0;
    const MAX: Self::Primitive = 23;
    type Primitive = i8;
}

#[derive(Clone, Copy)]
struct Secs(());
impl Bounds for Secs {
    const WHAT: &'static str = "seconds";
    const MIN: Self::Primitive = -5;
    const MAX: Self::Primitive = i64::MAX;
    type Primitive = i64;
}

impl Hour {
    pub const MIN: i8 = <Hour as Bounds>::MIN;
    pub const LEN: i128 = <Hour as Bounds>::MAX as i128 - Self::MIN as i128 + 1;
}

const HOURS: i128 = Hour::LEN * 2;

#[derive(Clone, Copy)]
pub struct Raw<B>(std::marker::PhantomData<B>);

impl<B, P> fmt::Debug for Raw<B>
where
    B: Bounds<Primitive = P>,
    P: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Raw").field("what", &B::WHAT).field("min", &B::MIN).field("max", &B::MAX).finish()
    }
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Hour(Raw<Hour>),
    Secs(Raw<Secs>),
}

fn describe<B: Bounds>() -> String {
    format!("{} {}..={}", B::WHAT, B::MIN, B::MAX)
}

const fn in_i8<B>(n: i64) -> bool
where
    B: Bounds<Primitive = i8>,
{
    (B::MIN as i64) <= n && n <= (B::MAX as i64)
}

fn main() {
    println!("{} {} {}", Hour::MIN, Hour::LEN, HOURS);
    println!("{} | {}", describe::<Hour>(), describe::<Secs>());
    println!("{:?} {:?} {:?}", Hour::check(5), Hour::check(24), Secs::check(-6));
    println!("{} {}", in_i8::<Hour>(23), in_i8::<Hour>(24));
    let k = [Kind::Hour(Raw(std::marker::PhantomData)), Kind::Secs(Raw(std::marker::PhantomData))];
    println!("{:?}", k);
    println!("{:#?}", k[0]);
}
