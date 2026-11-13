// expect: L0524
// rustc: allocators must be statics.
#[global_allocator]
fn f() {}

fn main() {
    println!("hi");
}
