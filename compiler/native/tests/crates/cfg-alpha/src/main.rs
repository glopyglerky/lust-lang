#[cfg(feature = "alpha")]
fn pick() -> i64 {
    11
}

#[cfg(not(feature = "alpha"))]
fn pick() -> i64 {
    22
}

fn main() {
    println!("cfg {}", pick());
}
