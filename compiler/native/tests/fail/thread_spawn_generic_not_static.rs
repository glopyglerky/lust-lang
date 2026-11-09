// expect: L0612
fn go<T: Send>(t: T) {
    std::thread::spawn(move || drop(t)).join().unwrap();
}
fn main() {
    go(5);
    println!("ok");
}
