// panic: index out of bounds
// A custom hook under panic=abort runs before the abort (custom: ...).
fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("custom: {}", info);
    }));
    let v = vec![1, 2, 3];
    println!("start");
    let x = v[10];
    println!("{}", x);
}
