// feature: ext.nix
// form: the thp_guard.rs prctl pair (get, set back to itself)
// cargo-dep: nix = { version = "=0.31.3", default-features = false, features = ["process", "signal"] }
use nix::sys::prctl::{get_thp_disable, set_thp_disable};
fn main() {
    let before = get_thp_disable().unwrap();
    set_thp_disable(before).unwrap();
    println!("{}", get_thp_disable().unwrap() == before);
}
