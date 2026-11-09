mod m;
mod other;
mod user;
fn main() {
    println!("{} {}", user::by_glob(), other::by_path());
}
