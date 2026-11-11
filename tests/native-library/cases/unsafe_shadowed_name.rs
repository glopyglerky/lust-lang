use left::unchecked_add as operation;

fn main() {
    let value = unsafe { operation(25) };
    {
        let operation: fn(i32, i32) -> i32 = left::combine;
        assert_eq!(operation(10, 1), 31);
    }
    println!("unsafe-shadow={}", value);
}
