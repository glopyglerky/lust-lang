const fn cf() -> i32 { 1 }
const C: i32 = cf() ;
const D: i32 = C + 1;
fn show() -> i32 {
    let x: i32 = "no"; x
}
fn main() {
    println!("{} {} {}", C, D, show());
}
