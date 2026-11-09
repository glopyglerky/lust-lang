use std::sync::Mutex;
fn main() {
    let m = Mutex::new(5);
    let g = m.lock().unwrap();
    let f = move || println!("{}", *g);
    f();
    drop(f);
    println!("{:?}", m);
    let g2 = m.lock().unwrap();
    let f2 = move || { let v: u8 = 1; v + (*g2 as u8) };
    println!("{} {:?}", f2(), m);
}
