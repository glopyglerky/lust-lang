// expect: L0511
fn main() {
    thread_local! { pub(in nowhere) static T: u32 = 1; }
    T.with(|t| println!("{}", t));
}
