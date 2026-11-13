// A collect whose target only a later call fixes (Vec<Rx<R>>) settles before flatten needs
// its items, as rustc's obligation pass does (ts-rust program.rs wait_jobs(receivers).flatten()).
struct Rx<R>(R);
fn send<R>(f: impl FnOnce() -> R) -> Rx<R> { Rx(f()) }
fn wait<R>(rs: Vec<Rx<R>>) -> Vec<R> { rs.into_iter().map(|r| r.0).collect() }
fn diags(i: u32) -> Vec<u32> { vec![i, i * 10] }
fn main() {
    let files = vec![1u32, 2, 3];
    let receivers = files.into_iter().map(|f| send(move || diags(f))).collect();
    let all: Vec<u32> = wait(receivers).into_iter().flatten().collect();
    println!("{all:?}");
}
