// A closure stored in an Option, then called out of it.
fn main() {
    let k = 2;
    let f = Some(move |x: i32| x * k);
    if let Some(g) = f {
        println!("{}", g(4));
    }
}
