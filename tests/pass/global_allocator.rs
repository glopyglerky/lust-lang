// #[global_allocator] with std's System: accepted, and the runtime
// allocator keeps serving allocations, so output matches rustc.
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() {
    let mut v: Vec<String> = Vec::new();
    for i in 0..50 {
        v.push(format!("n{i}"));
    }
    v.sort();
    println!("{} {} {} {:?}", v.len(), v[0], v[49], GLOBAL);
    let a = std::alloc::System;
    let b = a;
    let c = std::alloc::System::default();
    println!("{:?} {:?} {:?} {}", a, b, c, std::mem::size_of::<std::alloc::System>());
}
