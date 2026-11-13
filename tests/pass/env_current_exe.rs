// env::current_exe: an absolute path that exists (the running binary).
fn main() {
    let exe = std::env::current_exe().unwrap();
    println!("abs={} exists={} parent={}", exe.is_absolute(), exe.exists(), exe.parent().is_some());
    println!("os={} family={} arch={}", std::env::consts::OS, std::env::consts::FAMILY, std::env::consts::ARCH);
}
