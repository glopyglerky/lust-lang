use std::thread;
fn main() {
    let h = thread::spawn(|| { let t = thread::current(); (format!("{:?} {:?}", t.id(), t.name()), 5u8) });
    let h2 = thread::spawn(|| 1);
    println!("{}", h2.thread().id() == h2.thread().id());
    let id = format!("{:?}", h.thread().id());
    let (inner, v) = h.join().unwrap();
    println!("{} {} {} {:?} {:?}", id, inner, v, thread::current().id(), thread::current().name());
    println!("{:?}", thread::current());
    println!("{}", h2.join().unwrap());
}
