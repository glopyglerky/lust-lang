// Another module only: collections leaves the splice, time joins.
use std::time::Duration;

pub fn go() {
    let d = Duration::from_millis(2500);
    println!("{} {}", d.as_secs(), d.subsec_millis());
}
