// expect: L0612
// `impl Sum for Duration` needs user `Sum` impls, which lust lacks yet.
use std::time::Duration;
fn main() {
    let v = vec![Duration::from_secs(1), Duration::from_millis(500)];
    let total: Duration = v.iter().sum();
    println!("{:?}", total);
}
