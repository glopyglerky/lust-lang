// feature: threads.send_sync_bounds
// form: T: Send + Sync generic bounds
fn needs<T: Send + Sync>(x: T) -> T { x }
fn main() { println!("{}", needs(3)); }
