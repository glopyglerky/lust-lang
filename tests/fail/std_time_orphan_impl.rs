// expect: L0616
// rustc E0117: neither `Display` nor `Duration` is local.
use std::fmt;
use std::time::Duration;
impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}s", self.as_secs())
    }
}
fn main() {
    println!("{}", Duration::from_secs(2));
}
