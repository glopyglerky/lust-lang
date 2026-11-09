fn run<F: FnOnce() -> u32 + Send + 'static>(f: F) -> u32 { std::thread::spawn(f).join().unwrap() }
fn w() -> u32 { 5 }
fn main() { println!("{} {}", run(|| 7), run(w)); }
