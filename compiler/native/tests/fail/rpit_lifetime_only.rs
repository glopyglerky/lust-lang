// expect: L0110
// rustc E0224: at least one trait must be specified.
fn bad() -> impl 'static {
    5
}

fn main() {}
