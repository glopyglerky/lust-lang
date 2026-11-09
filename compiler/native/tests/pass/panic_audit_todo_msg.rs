// `todo!` with a message: rustc's prefix, then the message.
// panic: not yet implemented: part 2 of 3
fn step(i: usize) -> usize {
    if i < 2 {
        return i + 1;
    }
    todo!("part {} of {}", i, 3)
}

fn main() {
    let n = std::env::args().count();
    println!("{}", step(n - 1));
    println!("{}", step(n + 1));
}
