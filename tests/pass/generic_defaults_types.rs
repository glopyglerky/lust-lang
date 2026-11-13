// Generic parameter defaults on structs and enums: omitted trailing
// args take the defaults in types, a default may name earlier params,
// and a turbofish fixes the written params while the rest default.
#[derive(Debug)]
struct Pair<T, U = u8> {
    a: T,
    b: U,
}

impl<T, U: Default> Pair<T, U> {
    fn new(a: T) -> Self {
        Pair { a, b: U::default() }
    }
}

#[derive(Debug)]
struct Same<T, U = T>(T, U);

#[derive(Debug)]
enum Either<L, R = String> {
    Left(L),
    Right(R),
}

struct Arr<const N: usize = 3> {
    v: [u8; N],
}

struct Holder<P = Pair<i16>> {
    inner: P,
}

struct Unit<T = u32>(T);

impl Unit {
    fn get(&self) -> u32 {
        self.0
    }
}

fn sum(p: &Pair<i32>) -> i32 {
    p.a + p.b as i32
}

fn main() {
    let p: Pair<i32> = Pair { a: 1, b: 255 };
    let q = Pair::<i64>::new(7);
    let r = Pair::<&str> { a: "x", b: 9 };
    let s: Same<u16> = Same(3, 4);
    let e: Either<i32> = Either::Right("r".to_string());
    let f = Either::<u8>::Left(200);
    let arr: Arr = Arr { v: [1, 2, 3] };
    let h: Holder = Holder { inner: Pair { a: -2, b: 1 } };
    let u = Unit(41);
    println!("{:?} {:?} {:?} {:?}", p, q, r, s);
    println!("{:?} {:?} {}", e, f, arr.v.len());
    println!("{} {} {}", h.inner.a, sum(&p), u.get() + 1);
    // The default fixes `U = u8`: 255 + 1 overflows in u8 only.
    println!("{}", p.b.checked_add(1).is_none());
}
