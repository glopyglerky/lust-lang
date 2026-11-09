mod hidden;
mod api;

fn main() {
    let s = api::make();
    println!("reexport {}", s.v);
}
