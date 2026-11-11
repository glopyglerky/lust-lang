extern "C" fn twice(value: i32) -> i32 {
    value * 2
}

fn main() {
    println!("{}", twice(7));
}
