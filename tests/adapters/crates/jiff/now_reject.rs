// lust-reject: L0104
// The adapter has only the clock surface ts-rust uses; the real crate
// has `Timestamp::now`.
fn main() {
    let now = jiff::Timestamp::now();
    let _ = now;
    println!("ok");
}
