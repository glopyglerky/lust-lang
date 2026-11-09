// fs::read_link and Path::read_link (R24): the target, a capacity equal to
// the length (std shrinks its readlink buffer), and std's error kinds.
use std::path::Path;
fn main() {
    let dir = std::env::temp_dir().join(format!("rl_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("t.txt");
    std::fs::write(&target, "x").unwrap();
    let link = dir.join("l");
    std::process::Command::new("ln").arg("-s").arg("t.txt").arg(&link).status().unwrap();
    let r = std::fs::read_link(&link).unwrap();
    println!("{:?} {} {}", r, r.as_os_str().len(), r.capacity());
    let r2 = link.read_link().unwrap();
    println!("{}", r2.display());
    let long = "a".repeat(300);
    let l2 = dir.join("long");
    std::process::Command::new("ln").arg("-s").arg(&long).arg(&l2).status().unwrap();
    let r3 = std::fs::read_link(&l2).unwrap();
    println!("{} {}", r3.as_os_str().len(), r3.capacity());
    let c = std::fs::canonicalize(&link).unwrap();
    println!("{} {}", c == target, c.capacity() == c.as_os_str().len());
    match std::fs::read_link(&target) { Ok(_) => println!("ok?"), Err(e) => println!("{:?} {}", e.kind(), e) }
    match std::fs::read_link(dir.join("missing")) { Ok(_) => println!("ok?"), Err(e) => println!("{:?} {}", e.kind(), e) }
    match Path::new("a\0b").read_link() { Ok(_) => println!("ok?"), Err(e) => println!("{:?} {}", e.kind(), e) }
    std::fs::remove_dir_all(&dir).unwrap();
}
