#[path = "else/where.rs"]
mod w;

mod gostd;

mod outer {
    #[path = "deep.rs"]
    pub mod deep;
}

fn main() {
    println!("path {}", w::v() + outer::deep::v() + gostd::strconv::v());
}
