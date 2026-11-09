use std::thread;
fn main() {
    let mut x = 0;
    thread::scope(|s| { s.spawn(|| { x += 1; }); });
    x += 1;
    println!("{}", x);
}
