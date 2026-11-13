// capability: rejected: extern functions
// expect: L0810

extern "C" fn f(x: i32) -> i32 {
    x + 1
}

fn main() {
    println!("{}", f(1));
}
