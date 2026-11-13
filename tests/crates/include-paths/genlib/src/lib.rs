pub mod deep;

pub fn dep_pkg() -> &'static str {
    env!("CARGO_PKG_NAME")
}
