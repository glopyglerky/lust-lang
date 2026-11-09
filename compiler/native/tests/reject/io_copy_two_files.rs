// expect: L0641
// Valid Rust; std copies between two files in the kernel
// (copy_file_range), which lust does not model.
fn main() {
    let mut f = std::fs::File::open("/etc/hostname").unwrap();
    let mut o = std::fs::File::create(std::env::temp_dir().join("io_copy_two_files")).unwrap();
    println!("{}", std::io::copy(&mut f, &mut o).unwrap() > 0);
}
