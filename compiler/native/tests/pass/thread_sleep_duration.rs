use std::time::Duration;
use std::thread;
fn main() {
    let d = Duration::from_millis(1500);
    println!("{} {} {} {} {}", d.as_secs(), d.as_millis(), d.subsec_millis(), d.as_micros(), d.as_nanos());
    let e = Duration::new(2, 1_500_000_000);
    println!("{} {} {} {}", e.as_secs(), e.subsec_nanos(), e == Duration::from_nanos(3_500_000_000), Duration::default().is_zero());
    let big = Duration::from_secs(u64::MAX);
    println!("{} {}", big.as_millis(), big.as_nanos());
    let c = d; let f = c.clone();
    thread::sleep(Duration::from_millis(5));
    let h = thread::spawn(move || { thread::sleep(Duration::from_micros(300)); f.subsec_micros() });
    println!("{} {}", h.join().unwrap(), Duration::from_micros(1_000_001).as_secs());
}
