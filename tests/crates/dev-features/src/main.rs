#[cfg(feature = "heavy")]
fn pick() -> i64 {
    11
}

#[cfg(not(feature = "heavy"))]
fn pick() -> i64 {
    22
}

fn main() {
    let fast = if cfg!(feature = "fast") { 1 } else { 0 };
    println!("dev-features {} {}", pick(), fast);
}
