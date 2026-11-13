// lust-reject: L0112
// Upstream prints a Timestamp as RFC 3339 text; the adapter has no
// Display for it, so lust rejects the format.
use std::time::SystemTime;
fn main() {
    let ts = jiff::Timestamp::try_from(SystemTime::UNIX_EPOCH).unwrap();
    println!("{ts}");
}
