const fn cf() -> i32 { 11 }
const A: i32 = cf();
const B: i32 = A * 2;
const C: i32 = B + A;
const D: i32 = C - 1;
fn main() {
    println!("{} {} {} {}", A, B, C, D);
}
