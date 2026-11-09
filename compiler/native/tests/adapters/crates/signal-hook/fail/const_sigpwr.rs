// expect: L0101
// rustc E0425: upstream re-exports libc's portable signal list only.
fn main() {
    println!("{}", signal_hook::consts::SIGPWR);
}
