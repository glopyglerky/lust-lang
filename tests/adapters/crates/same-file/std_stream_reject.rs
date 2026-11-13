// lust-reject: L0104
// same-file 1.0.6: Handle::stdout wraps fd 1 in a File it must
// not close. Lust std has no from-raw-fd File form yet, so the
// adapter does not have the std stream constructors (README).
// Cargo builds it fine.
use same_file::Handle;

fn main() {
    let out = Handle::stdout().unwrap();
    println!("{}", out.dev() > 0);
}
