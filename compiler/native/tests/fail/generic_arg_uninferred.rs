// expect: L0610
// rustc E0282: nothing fixes `T`, so the call needs a turbofish.
fn f<T>() -> u8 {
    1
}

fn main() {
    println!("{}", f());
}
