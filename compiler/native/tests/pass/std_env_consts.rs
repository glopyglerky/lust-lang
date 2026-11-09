// `std::env::consts` (ts-rust fanotify_available reads `consts::OS`): the
// target's OS, family, and arch, by full path and through a `use`.
use std::env::consts;
const OS: &str = std::env::consts::OS;
fn main() {
    println!("{} {} {}", OS, consts::FAMILY, consts::ARCH);
    if std::env::consts::OS != "linux" {
        println!("other");
    }
    println!("{}", consts::OS.len());
}
