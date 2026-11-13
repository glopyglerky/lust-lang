// expect: L0102
// `Cell::get` as a callable still needs `Cell` in scope (E0433).
thread_local! {
    static N: std::cell::Cell<u8> = const { std::cell::Cell::new(1) };
}
fn main() {
    println!("{}", N.with(Cell::get));
}
