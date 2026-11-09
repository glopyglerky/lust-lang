// A built-in array index inside a call argument: the location is the
// index expression, not the call.
// panic: index out of bounds: the len is 3 but the index is 5
fn show(x: u8) -> u8 {
    x
}

fn main() {
    let n = std::env::args().count();
    let a = [7u8, 8, 9];
    println!("{}", show(a[n]));
    println!("{}", show(a[n + 4]));
}
