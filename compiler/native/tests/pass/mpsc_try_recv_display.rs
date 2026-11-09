use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel::<i32>();
    println!("{}", rx.try_recv().unwrap_err());
    drop(tx);
    println!("{}", rx.try_recv().unwrap_err());
    println!("{}", rx.recv().unwrap_err());
}
