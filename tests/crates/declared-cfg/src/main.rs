#[cfg(checked_build)]
compile_error!("a declaration must not enable a flag");
#[cfg(alternate_build)]
compile_error!("all declared names remain false");
#[cfg(mode = "wide")]
compile_error!("declared values are not enabled values");
fn main() {
    assert!(!cfg!(checked_build));
    assert!(!cfg!(alternate_build));
    assert!(!cfg!(mode = "wide"));
    println!("declared and inactive");
}
