const fn inner() -> i32 { "bad" }
const fn outer() -> i32 { inner() * 3 }
const A: i32 = outer();
const B: i32 = A + 1;
const C: [i32; 2] = [A, B];
fn main() {
    println!("{} {} {:?}", A, B, C);
}
