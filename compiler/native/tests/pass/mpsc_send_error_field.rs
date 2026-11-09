// mpsc::SendError's field .0 (ts-rust execute/build/build_task.rs:453 send(v).err().map(|err| err.0)).
use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel::<String>();
    drop(rx);
    let e = tx.send(String::from("y")).err().map(|e| e.0);
    println!("{:?}", e);
    if let Err(err) = tx.send(String::from("z")) {
        let task = err.0;
        println!("back {}", task);
    }
    let r = tx.send(String::from("w"));
    match r { Err(e) => { let s: &String = &e.0; println!("{} {}", s.len(), e); } Ok(()) => {} }
    let (t2, r2) = mpsc::channel::<Vec<u8>>();
    drop(r2);
    let v = t2.send(vec![1, 2]).unwrap_err();
    println!("{:?} {:?}", v.0.len(), v);
}
