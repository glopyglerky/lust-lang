const fn cf() -> i32 { 9 }
const C: i32 = cf() + "x";
const D: i32 = C + 1;
fn show() -> i32 {
    D * 2
}
fn main() {
    println!("{} {} {}", C, D, show());
}
