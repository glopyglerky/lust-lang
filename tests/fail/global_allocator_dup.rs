// expect: L0524
// rustc: cannot define multiple global allocators.
#[global_allocator]
static A: std::alloc::System = std::alloc::System;
#[global_allocator]
static B: std::alloc::System = std::alloc::System;

fn main() {
    println!("hi");
}
