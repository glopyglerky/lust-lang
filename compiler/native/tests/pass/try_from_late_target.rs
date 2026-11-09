// `try_into` whose target a later use fixes (`.unwrap()`, `.ok()` at
// a return type, `?`), scalar `TryFrom` where clauses on user impls,
// and a user impl picked for an integer literal argument (fx-tryfrom).
fn narrow(x: i64) -> Option<u8> {
    x.try_into().ok()
}

fn later(x: u64) -> Result<u32, std::num::TryFromIntError> {
    let v = x.try_into()?;
    let w: u16 = v;
    Ok(w as u32 + 1)
}

#[derive(Debug)]
struct Wrap<T>(T);
impl<T: TryFrom<i64>> TryFrom<i64> for Wrap<T> {
    type Error = &'static str;
    fn try_from(p: i64) -> Result<Self, Self::Error> {
        T::try_from(p).map(Wrap).map_err(|_| "range")
    }
}

trait Show {
    fn show(&self) -> String;
}
struct W<T>(T);
impl<T: TryFrom<i64> + Copy + std::fmt::Debug> Show for W<T>
where
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    fn show(&self) -> String {
        format!("{:?} {:?}", self.0, T::try_from(300))
    }
}

#[derive(Debug)]
struct Even(u32);
impl TryFrom<u32> for Even {
    type Error = String;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        if v % 2 == 0 { Ok(Even(v)) } else { Err(format!("{} is odd", v)) }
    }
}

fn main() {
    let n: u16 = 3u32.try_into().unwrap();
    let m: i8 = (-3i64).try_into().expect("fits");
    println!("{} {}", n, m);
    println!("{:?} {:?} {:?}", narrow(3), narrow(-3), narrow(256));
    println!("{:?} {:?}", later(5), later(70000));
    println!("{:?} {:?} {:?}", Wrap::<u8>::try_from(3i64), Wrap::<u8>::try_from(300i64), Wrap::<i16>::try_from(300i64));
    println!("{} | {}", W(3u8).show(), W(3u16).show());
    println!("{:?} {:?}", Even::try_from(4), Even::try_from(5));
}
