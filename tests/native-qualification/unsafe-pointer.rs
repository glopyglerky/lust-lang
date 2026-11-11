fn twice(value: i32) -> i32 {
    value * 2
}

fn main() {
    let value = 7;
    let pointer = &value as *const i32;
    let loaded = unsafe { *pointer };
    println!("{}", twice(loaded));
}
