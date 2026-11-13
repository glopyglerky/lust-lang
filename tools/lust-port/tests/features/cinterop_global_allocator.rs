// feature: cinterop.global_allocator
// form: a #[global_allocator] static, here std's own System allocator
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;
fn main() {
    println!("hi");
}
