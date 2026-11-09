// expect: L0612
fn go<T: Send + Default>() {
    std::thread::spawn(|| T::default()).join().unwrap();
}
fn main() {
    go::<i32>();
    println!("ok");
}
