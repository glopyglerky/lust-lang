const fn inner() -> i32 { 4 }
const fn outer() -> i32 { inner() * "bad" }
const A: i32 = outer();
const B: i32 = A + 1;
const C: [i32; 2] = [A, B];
fn main() {
    println!("{} {} {:?}", A, B, C);
}
