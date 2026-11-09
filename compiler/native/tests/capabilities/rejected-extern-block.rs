// capability: rejected: extern blocks
// expect: L0810

unsafe extern "C" {
    safe fn abs(x: i32) -> i32;
}

fn main() {
    println!("{}", abs(-3));
}
