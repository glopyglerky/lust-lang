// panic: attempt to calculate the remainder with a divisor of zero
// `next_multiple_of(0)` panics on the remainder inside core, as in
// rustc (TRIAGE T8).
fn read<T>(v: T) -> T {
    v
}

fn main() {
    let z = read(1usize) - 1;
    println!("{}", read(5usize).next_multiple_of(z));
    println!("unreachable");
}
