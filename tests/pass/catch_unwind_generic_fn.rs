// `catch_unwind(AssertUnwindSafe(f))` with `f` a type param bounded by
// `FnOnce() -> R` (tsrust T17 gopanic.rs `go_wait_group_task`).
fn task<R>(f: impl FnOnce() -> R) -> R {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(_payload) => std::process::exit(2),
    }
}
fn task2<F: FnMut() -> u8>(f: F) -> u8 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(0)
}
fn main() {
    println!("{} {}", task(|| 5), task2(|| 6));
    println!("{}", task(|| String::from("s")));
}
