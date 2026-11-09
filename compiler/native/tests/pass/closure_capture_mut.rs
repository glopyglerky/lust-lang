// Mutation forces a unique borrow and makes the closure FnMut.
fn main() {
    let mut count = 0u32;
    let mut log: Vec<u32> = Vec::new();
    let mut bump = |by: u32| {
        count += by;
        log.push(count);
    };
    bump(2);
    bump(5);
    println!("{} {:?}", count, log);
}
