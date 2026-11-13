// The root file's cfg facts come from the session target (here the
// host), the same table every other file of a crate session sees.
fn main() {
    println!(
        "{} {} {} {} {} {}",
        cfg!(target_os = "linux"),
        cfg!(target_family = "unix"),
        cfg!(target_pointer_width = "64"),
        cfg!(target_endian = "little"),
        cfg!(target_has_atomic = "64"),
        cfg!(target_env = "gnu"),
    );
    println!("{}", cfg!(any(windows, target_arch = "wasm32")));
}
