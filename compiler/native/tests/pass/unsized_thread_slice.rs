use std::sync::Arc; use std::thread;
fn main() { let data: Arc<[u32]> = Arc::from(vec![1, 2, 3, 4]); let hs: Vec<_> = (0..3).map(|i| { let d = Arc::clone(&data); thread::spawn(move || d.iter().map(|x| x * i).sum::<u32>()) }).collect(); let t: u32 = hs.into_iter().map(|h| h.join().unwrap()).sum(); println!("{t} {}", Arc::strong_count(&data)); }
