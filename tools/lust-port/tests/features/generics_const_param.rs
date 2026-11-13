// feature: generics.const_param
// form: <const N: usize> on a fn and a struct, as in ts-rust checker helpers
fn sum<const N: usize>(a: [i32; N]) -> i32 { a.into_iter().sum() }
struct Buf<const N: usize> { data: [i32; N] }
fn main() { let b = Buf { data: [4, 5] }; println!("{} {}", sum([1, 2, 3]), b.data[0]); }
