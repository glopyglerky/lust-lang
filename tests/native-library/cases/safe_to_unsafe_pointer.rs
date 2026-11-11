fn main() {
    let operation: unsafe fn(i32, i32) -> i32 = left::combine;
    println!("safe-to-unsafe={}", unsafe { operation(10, 1) });
}
