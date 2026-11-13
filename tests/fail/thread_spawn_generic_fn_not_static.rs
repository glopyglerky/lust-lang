// expect: L0612
fn go<F: FnOnce() -> i32 + Send>(f: F) {
    std::thread::spawn(f).join().unwrap();
}
fn main() {
    go(|| 1);
    println!("ok");
}
