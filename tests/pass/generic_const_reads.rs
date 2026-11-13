// Consts, statics, and associated consts read inside generic
// bodies: the const tables are built before the template
// checks, so every read below answers as it does in a plain
// body (values for trait associated consts come from the
// confirmed impl, else the trait default).
const SECRET: i32 = 12;
static ST: i32 = 5;

struct S;
impl S {
    pub const PUBC: i32 = 3;
    const SEC: i32 = 4;
    const VIA: i32 = Self::SEC + 1;
    fn get<T>(&self, t: T) -> i32 {
        let _ = &t;
        Self::SEC
    }
}
const VIA_ASSOC: i32 = S::PUBC + 30;

trait Tr {
    const C: i32 = 7;
    fn via_self(&self) -> i32 {
        Self::C
    }
}
trait Need {
    const D: i32;
    fn get(&self) -> i32;
}
struct Q;
impl Tr for Q {}
struct R;
impl Tr for R {}
impl Need for R {
    const D: i32 = 8;
    fn get(&self) -> i32 {
        Self::D
    }
}

fn free_and_static<T>(t: T) -> i32 {
    let _ = &t;
    let f = || ST + S::PUBC;
    SECRET + f()
}
fn own_module<T>(t: T) -> i32 {
    let _ = &t;
    S::SEC + S::VIA + VIA_ASSOC
}
fn from_bound<T: Need>(t: T) -> i32 {
    let _ = &t;
    T::D + <T as Need>::D
}
fn concrete<T>(t: T) -> i32 {
    let _ = &t;
    <R as Tr>::C + Q::C + <R as Need>::D
}
fn shapes<T>(t: T, x: i32) -> i32 {
    let _ = &t;
    let a: [i32; S::PUBC as usize] = [1, 2, 3];
    let hit = match x {
        SECRET => 1,
        S::PUBC..=S::SEC => 2,
        _ => 0,
    };
    const LOCAL: i32 = 10;
    a.len() as i32 + hit + LOCAL
}

mod inner {
    const OWN: i32 = 20;
    pub fn read<T>(t: T) -> i32 {
        let _ = &t;
        OWN + super::SECRET
    }
}

fn main() {
    println!("{}", free_and_static(0));
    println!("{}", own_module(0));
    println!("{}", S.get(0));
    println!("{}", from_bound(R));
    println!("{}", concrete(0));
    println!("{} {}", R.via_self(), R.get());
    println!("{} {}", shapes(0, 12), shapes(0, 3));
    println!("{}", inner::read(0));
}
