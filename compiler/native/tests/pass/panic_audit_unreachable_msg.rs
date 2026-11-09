// `unreachable!` with a message, and `panic!("")` style empty messages.
// panic: internal error: entered unreachable code: state Some(3)
fn main() {
    let n = std::env::args().count();
    let state = if n > 0 { Some(n + 2) } else { None };
    match state {
        None => println!("none"),
        Some(1) => println!("one"),
        s => unreachable!("state {:?}", s),
    }
}
