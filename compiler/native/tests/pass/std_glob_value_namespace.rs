// A std glob brings `Weak`, `Mutex`, and `Duration` as types only, so in
// the value namespace the names still reach the outer items. Globs are
// ambiguous per namespace: io and fmt both have `Error`, but only fmt's
// is a value.
#![allow(non_snake_case)]
fn Weak() -> u8 {
    2
}
fn Mutex() -> u8 {
    3
}
const Rc: u8 = 4;
fn main() {
    use std::rc::*;
    use std::sync::*;
    println!("{} {} {}", Weak(), Mutex(), Rc);
    let r: Rc<u8> = Rc::new(5);
    let m = Mutex::new(6);
    println!("{} {}", r, m.lock().unwrap());
    {
        use std::time::*;
        fn Duration() -> u8 {
            7
        }
        println!("{} {}", Duration(), Instant::now().elapsed().as_secs() < 100);
    }
    let Weak = 8;
    println!("{}", Weak);
}
