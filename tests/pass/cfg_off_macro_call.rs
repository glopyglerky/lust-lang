#[cfg(target_family = "wasm")]
include!(concat!(env!("OUT_DIR"), "/nope.rs"));

#[cfg(target_family = "wasm")]
compile_error!("must not fire on this target");

fn f() -> i64 {
    #[cfg(target_family = "wasm")]
    compile_error!("also off");
    5
}

fn main() {
    println!("{}", f());
}
