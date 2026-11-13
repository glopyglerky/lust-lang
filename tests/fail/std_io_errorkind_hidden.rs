// expect: L0104
// Lust's source marks std's unstable io::ErrorKind kinds with a `__`
// prefix; user code cannot name them (rustc has no such variant).
use std::io::ErrorKind;
fn main() {
    let k = ErrorKind::__Uncategorized;
    if let ErrorKind::__InProgress = k {
        println!("x");
    }
}
