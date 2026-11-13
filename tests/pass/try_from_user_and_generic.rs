// User `TryFrom` impls, the `TryInto` blanket through them, the
// `TryFrom` blanket through a user `From`, generic bounds in both
// directions, and `?` into a custom error (fx-tryfrom).
#[derive(Debug)]
struct Small(u8);
impl TryFrom<u16> for Small {
    type Error = String;
    fn try_from(v: u16) -> Result<Self, String> {
        if v > 255 { Err("big".to_string()) } else { Ok(Small(v as u8)) }
    }
}
#[derive(Debug)]
struct Wrap(i64);
impl From<i32> for Wrap {
    fn from(v: i32) -> Wrap {
        Wrap(v as i64 * 2)
    }
}
fn conv<U: TryFrom<u64>>(v: u64) -> Option<U> {
    U::try_from(v).ok()
}
fn conv_into<U: TryInto<u8>>(v: U) -> u8 {
    v.try_into().unwrap_or(7)
}
fn conv_err<T: TryFrom<u32, Error = std::num::TryFromIntError>>(v: u32) -> Result<T, std::num::TryFromIntError> {
    T::try_from(v)
}
#[derive(Debug)]
enum MyErr {
    Num(std::num::TryFromIntError),
}
impl From<std::num::TryFromIntError> for MyErr {
    fn from(e: std::num::TryFromIntError) -> MyErr {
        MyErr::Num(e)
    }
}
fn g(v: i32) -> Result<u8, MyErr> {
    Ok(v.try_into()?)
}
fn main() {
    println!("{}", Small::try_from(300u16).unwrap_err());
    let r: Result<Small, String> = 300u16.try_into();
    println!("{}", r.unwrap_err());
    let ok: Small = 3u16.try_into().unwrap();
    println!("{}", ok.0);
    let w: Result<Wrap, std::convert::Infallible> = Wrap::try_from(21i32);
    println!("{:?}", w);
    let w2: Result<Wrap, std::convert::Infallible> = Wrap::try_from(Wrap(9));
    println!("{:?}", w2);
    println!("{:?} {:?}", conv::<u32>(5), conv::<u8>(300));
    println!("{} {}", conv_into(300i32), conv_into(3i32));
    println!("{:?} {:?}", conv_err::<u8>(300), conv_err::<u8>(3));
    println!("{:?} {:?} {:?}", g(300), g(-1), g(3));
}
