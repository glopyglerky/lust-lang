// `std::process::exit` is a source std fn (S5): its body lives in
// library/std/src/process.rs, checked by the std loader and spliced
// into this compile. Output before the call is flushed, the exit code
// is the argument, and nothing after the call runs. The exit fn also
// works in a never-typed value position.
fn pick(yes: bool) -> i32 {
    if yes {
        5
    } else {
        std::process::exit(9)
    }
}

fn main() {
    println!("picked {}", pick(true));
    println!("about to exit");
    std::process::exit(7);
    println!("unreachable");
}
