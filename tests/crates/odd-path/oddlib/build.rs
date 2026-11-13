fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-env=LIB_TEXT=lib-from-script");
}
