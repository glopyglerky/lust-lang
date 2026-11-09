// A str range whose start passes its end.
// panic: byte range starts at 4 but ends at 2
fn main() {
    let n = std::env::args().count();
    let s = "hello";
    println!("{}", &s[n..=n + 2]);
    println!("{}", &s[n + 3..n + 1]);
}
