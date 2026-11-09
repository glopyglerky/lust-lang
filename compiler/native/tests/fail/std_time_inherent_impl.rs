// expect: L0616
// rustc E0116: no inherent impl on a std type.
use std::time::Duration;
impl Duration {
    fn halves(self) -> Duration {
        self / 2
    }
}
fn main() {
    println!("{:?}", Duration::from_secs(2).halves());
}
