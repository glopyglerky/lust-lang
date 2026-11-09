// expect: L0612
// A raw pointer is neither Send nor Sync, so it cannot move into a
// spawned thread.
fn main() {
    let x = 4;
    let p: *const i32 = &x;
    let h = std::thread::spawn(move || p.is_null());
    println!("{}", h.join().unwrap());
}
