#[cfg(any())]
fn chosen() -> bool {
    left::function_that_does_not_exist()
}

#[cfg(not(any()))]
fn chosen() -> i32 {
    left::combine(10, 1)
}

fn main() {
    println!("conditional={}", chosen());
}
