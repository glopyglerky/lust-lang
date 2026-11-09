// expect: L0112
// rustc E0277: `u32` does not implement `GlobalAlloc`.
#[global_allocator]
static GLOBAL: u32 = 3;

fn main() {
    println!("hi");
}
