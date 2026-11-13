// reject: contrast case for sem_cfg_builtins: with cfg(all()) the compile_error! does fire.
// rustc-error: this cfg is on
#[cfg(all())]
fn boom() { compile_error!("this cfg is on: must fire"); }
fn main() {}
