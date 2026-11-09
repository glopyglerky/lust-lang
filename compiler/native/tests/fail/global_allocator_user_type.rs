// expect: L0112
// rustc E0277: a user struct with the allocator's name still does
// not implement `GlobalAlloc`.
struct Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    println!("hi");
}
