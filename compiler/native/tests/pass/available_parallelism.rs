// thread::available_parallelism as std 1.99 computes it on Linux: the
// affinity count capped by the cgroup quota. Both builds run on the same
// box, so the counts match. A file that never names `io` still types
// the Err arm (io::Error comes from the platform crate).
fn main() {
    let a = std::thread::available_parallelism();
    println!("{:?}", a);
    match a {
        Ok(n) => println!("{}", n.get()),
        Err(e) => println!("err {} {:?}", e, e.kind()),
    }
    if let Err(e) = std::thread::available_parallelism() {
        println!("{e}");
    }
}
