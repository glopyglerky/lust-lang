// expect: L0302
// rustc E0384: an arm after a catch-all is still borrow-checked (false edge).
fn f(v: u8) -> u8 {
    match v {
        _ => 0,
        x => { x += 1; x }
    }
}
fn main() { println!("{}", f(3)); }
