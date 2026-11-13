// expect: L0110
// rustc E0308: upstream `Stat::st_mtime_nsec` is a `u64`.
fn main() {
    let st = rustix::fs::fstat(rustix::stdio::stdin()).unwrap();
    let ns: i64 = st.st_mtime_nsec;
    println!("{}", ns >= 0);
}
