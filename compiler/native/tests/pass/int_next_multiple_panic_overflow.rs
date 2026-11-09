// panic: attempt to add with overflow
// `next_multiple_of` past the type maximum panics inside core, as in
// rustc (TRIAGE T8).
fn read<T>(v: T) -> T {
    v
}

fn main() {
    println!("{}", read(240u8).next_multiple_of(16));
    println!("{}", read(250u8).next_multiple_of(16));
    println!("unreachable");
}
