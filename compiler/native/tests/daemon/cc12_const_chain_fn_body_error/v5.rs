struct W(i32);
const fn cf() -> i32 {
    let x: i32 = 25;
    
    x
}
const C: i32 = cf();
const D: i32 = C + 1;
static S: i32 = D * 2;
fn main() {
    println!("{} {} {}", C, D, S);
}
