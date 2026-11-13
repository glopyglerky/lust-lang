// feature: std.runtime
// form: Duration values and arithmetic, as in ts-rust timing code
use std::time::Duration;
fn main() {
    let d = Duration::from_millis(1500) + Duration::from_millis(500);
    println!("{} {}", d.as_secs(), d.subsec_millis());
}
