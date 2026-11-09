// `mem::drop::<T>` with a turbofish as a value: the closure the parser
// makes calls `drop::<String>`, so the argument must be a String.
fn main() {
    let f = std::mem::drop::<String>;
    f(String::from("x"));
    println!("ok");
}
