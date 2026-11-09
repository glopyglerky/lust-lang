// feature: core.std_basics
// form: fs metadata and env lookup
fn main() {
    let is_file = match std::fs::metadata("Cargo.toml") { Ok(m) => m.is_file(), Err(_) => false };
    println!("{} {}", is_file, std::env::var_os("PATH").is_some());
}
