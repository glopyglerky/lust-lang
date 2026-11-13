// `thread::sleep` (W-THREADS) takes the std source `Duration` (W-STD-C):
// `std::time::Duration` names it once std::time is spliced.
use std::time::{Duration, Instant};
fn main() {
    let t = Instant::now();
    std::thread::sleep(Duration::from_millis(30));
    let e = t.elapsed();
    println!("{} {}", e >= Duration::from_millis(30), e < Duration::from_secs(5));
    let h = std::thread::spawn(|| { std::thread::sleep(Duration::new(0, 5_000_000)); 7 });
    println!("{:?}", h.join().unwrap());
}
