// Overflow panics with overflow-checks on. Stdout before the panic must still appear.
fn add(a: u8, b: u8) -> u8 {
    a + b
}

fn main() {
    println!("before");
    let total = add(200, 100);
    println!("after {}", total);
}
