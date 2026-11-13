// The fix, plus a std module and a platform module together.
use std::io::Write;
use std::time::Duration;

pub fn go() {
    let x: Option<u8> = Some(3);
    let d = Duration::from_secs(2);
    writeln!(std::io::stdout(), "{} {}", x.unwrap_or(0), d.as_secs()).unwrap();
}
