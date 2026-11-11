fn main() {
    let operation: unsafe fn(i32) -> i32 = left::unchecked_add;
    println!("unsafe-pointer={}", unsafe { operation(25) });
}
