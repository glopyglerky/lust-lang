use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
const CHUNKS: usize = 4;
static SLOTS: [OnceLock<usize>; CHUNKS] = [const { OnceLock::new() }; CHUNKS];
static TOTAL: Mutex<u64> = Mutex::new(0);
fn bind(name: &str, text: &str) -> (String, usize) { (name.to_string(), text.len()) }
fn main() {
    let files: Vec<(String, String)> = (0..8).map(|i| (format!("f{i}.ts"), "let x = 1;".repeat(i + 1))).collect();
    let symbols = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    thread::scope(|s| {
        for k in 0..CHUNKS {
            let tx = tx.clone();
            let symbols = Arc::clone(&symbols);
            let files = &files;
            s.spawn(move || {
                let mut n = 0;
                for i in (k * 2)..(k * 2 + 2) {
                    let r = bind(&files[i].0, &files[i].1);
                    symbols.lock().unwrap().push(r.0.clone());
                    *TOTAL.lock().unwrap() += r.1 as u64;
                    tx.send(r).unwrap();
                    n += 1;
                }
                SLOTS[k].get_or_init(|| n);
            });
        }
    });
    drop(tx);
    let mut results = Vec::new();
    while let Ok(r) = rx.recv() { results.push(r); }
    results.sort();
    let mut syms = symbols.lock().unwrap().clone();
    syms.sort();
    println!("{:?}", results);
    println!("{:?} {}", syms, TOTAL.lock().unwrap());
    for s in SLOTS.iter() { print!("{:?} ", s.get()); }
    println!();
}
