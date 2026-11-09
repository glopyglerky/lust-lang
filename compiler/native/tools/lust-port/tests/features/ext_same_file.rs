// feature: ext.same_file
// form: is_same_file with the unwrap_or(false) fallback, as compile.rs writes it
// cargo-dep: same-file = "=1.0.6"
use std::path::Path;
fn os_path(s: &str) -> &Path {
    Path::new(s)
}
fn main() {
    let dir = std::env::temp_dir().join("lust_port_same_file");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("x.exe");
    let b = dir.join("y.exe");
    std::fs::write(&a, "x").unwrap();
    std::fs::write(&b, "x").unwrap();
    let dotf = a.to_str().unwrap().to_string();
    let f = b.to_str().unwrap().to_string();
    println!("{}", !same_file::is_same_file(os_path(&dotf), os_path(&f)).unwrap_or(false));
    println!("{}", same_file::is_same_file(os_path(&dotf), os_path(&dotf)).unwrap_or(false));
    let _ = std::fs::remove_dir_all(&dir);
}
