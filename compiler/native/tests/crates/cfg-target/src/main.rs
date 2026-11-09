#[cfg(unix)]
fn platform() -> i64 {
    1
}

#[cfg(windows)]
fn platform() -> i64 {
    2
}

fn main() {
    println!("target {}", platform());
}
