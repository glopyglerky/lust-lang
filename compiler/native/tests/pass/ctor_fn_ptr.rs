// Tuple struct and variant constructors, `Some`, `Ok`, and `Err` coerce
// to fn pointers (ts-rust parsing_helpers.rs passes `JsxEmit` as a
// `fn(i32) -> T`): large returns, niche and explicit-tag enums, zero-sized
// fields, generic ctors, casts, and if/match/array joins of two ctors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JsxEmit(pub i32);
#[derive(Debug)]
struct Pair(u8, String);
#[derive(Debug)]
struct Big(u64, u64, u64, u64, u64);
#[derive(Debug)]
struct W<T>(T, ());
#[derive(Debug)]
enum E { A(i32), B(i32), C(String) }
#[repr(u8)]
#[derive(Debug)]
enum R { X(u16) = 3, Y(u16) = 9 }
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn float_or_int32_to_flag<T>(x: i32, from_i32: fn(i32) -> T) -> T {
    from_i32(x)
}
fn mk<T>() -> fn(T) -> Option<T> {
    Some
}
fn main() {
    let n = std::env::args().count() as i32;
    println!("{:?}", float_or_int32_to_flag(3, JsxEmit));
    let g: fn(u8, String) -> Pair = Pair;
    let h: fn(u64, u64, u64, u64, u64) -> Big = Big;
    let w: fn(u16, ()) -> W<u16> = W;
    let c: fn(String) -> E = E::C;
    let ok: fn(u8) -> Result<u8, String> = Ok;
    let er: fn(String) -> Result<u8, String> = Err;
    println!("{:?} {:?} {:?} {:?}", g(2, "x".into()), h(1, 2, 3, 4, 5), w(7, ()), c("c".into()));
    println!("{:?} {:?} {:?} {:?}", ok(6), er("e".into()), mk::<u8>()(1), mk::<String>()("s".into()));
    let x = R::X as fn(u16) -> R;
    println!("{:?} {:?}", x(300), (R::Y as fn(u16) -> R)(9));
    let f = if n > 1 { E::A } else { E::B };
    let k = match n { 1 => E::A, 2 => E::B, _ => E::A };
    let arr = [E::A, E::B, E::A];
    println!("{:?} {:?} {:?}", f(1), k(2), arr.iter().map(|f| f(3)).collect::<Vec<_>>());
    let d: fn(i32) -> D = D;
    let a = d(1);
    let b = d(2);
    drop(a);
    println!("mid");
    drop(b);
    let v: Vec<fn(i32) -> JsxEmit> = vec![JsxEmit; 2];
    println!("{}", v.iter().all(|f| f(4) == JsxEmit(4)));
}
