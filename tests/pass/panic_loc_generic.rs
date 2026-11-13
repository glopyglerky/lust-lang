// panic: panic_loc_generic.rs:7:5
// A panic in a generic instance body reports the line in the template.
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    // The overflow is on line 7, column 5.
    let x = a;
    let y = b;
    x + y
}

fn main() {
    println!("{}", add(1u16, 2));
    println!("{}", add(200u8, 100));
}
