// expect: L0104
// rustc E0599: the adapter's `Arg` hook is not an upstream method.
use rustix::path::Arg;

fn main() {
    println!("{}", "/tmp".__lust_path_bytes().len());
}
