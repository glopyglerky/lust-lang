mod store;
fn main() {
    let total = store::put(40);
    println!("{total}");
}
