use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel();
    for i in 0..5 { tx.send(i).unwrap(); }
    drop(tx);
    let firsts: Vec<i32> = rx.iter().take(2).collect();
    let rest: Vec<(usize, i32)> = rx.iter().enumerate().collect();
    println!("{:?} {:?}", firsts, rest);
}
