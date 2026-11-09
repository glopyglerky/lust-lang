// expect: L0524
// rustc: malformed `global_allocator` attribute input.
#[global_allocator = "x"]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() {
    println!("hi");
}
