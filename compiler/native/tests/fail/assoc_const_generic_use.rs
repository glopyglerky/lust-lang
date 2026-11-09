// expect: L0104
struct G<T>(T);
impl G<i32> { const C: u8 = 1; }
fn f<T>() -> u8 { G::<T>::C }
fn main() {
    println!("{}", f::<i32>());
}
