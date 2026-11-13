// `unimplemented!` says "not implemented", not `todo!`'s text.
// panic: not implemented: mode 7
fn run(mode: u32) -> u32 {
    match mode {
        0 => 10,
        m => unimplemented!("mode {m}"),
    }
}

fn main() {
    let n = std::env::args().count() as u32;
    println!("{}", run(n - 1));
    println!("{}", run(n + 6));
}
