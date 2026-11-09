// std::fs (W-SHIM): writes, reads, directories, metadata, listings,
// renames, and removal in a fresh scratch directory, with the exact
// io::Error values of each failure.
use std::fs;
use std::path::Path;

fn show<T: std::fmt::Debug>(label: &str, r: std::io::Result<T>) {
    match r {
        Ok(v) => println!("{} ok {:?}", label, v),
        Err(e) => println!("{} err {} | {:?} | {:?}", label, e, e, e.kind()),
    }
}

fn main() {
    let root = std::env::temp_dir().join(format!("lust_std_fs_{}", std::process::id()));
    show("setup", fs::create_dir_all(&root));
    show("cd", std::env::set_current_dir(&root));

    show("write", fs::write("f.txt", "hello"));
    show("write2", fs::write("g.bin", vec![1u8, 2, 3]));
    show("write3", fs::write(Path::new("h.txt"), String::from("x\ny\n")));
    show("write4", fs::write("i.txt", &b"bytes"[..]));
    show("read", fs::read_to_string("f.txt"));
    show("readb", fs::read("g.bin"));
    show("mkdirs", fs::create_dir_all("d/e/f"));
    show("mkdirs2", fs::create_dir_all("d/e"));
    show("mkdir", fs::create_dir("d"));
    show("mkdirs3", fs::create_dir_all("f.txt/x"));
    show("mkdirs4", fs::create_dir_all(""));
    match fs::metadata("f.txt") { Ok(m) => println!("{:?} {:?} {} {} {} {}", m.file_type(), m.permissions(), m.len(), m.is_file(), m.is_dir(), m.permissions().readonly()), Err(e) => println!("{}", e) }
    match fs::metadata("d") { Ok(m) => println!("{:?} {:?}", m.file_type(), m.permissions()), Err(e) => println!("{}", e) }
    show("read_dir_missing", fs::read_to_string("nope"));
    show("read_dir_isdir", fs::read_to_string("d"));
    show("bad", fs::write("bad.txt", vec![0xffu8, 0xfe]));
    show("badread", fs::read_to_string("bad.txt"));
    show("rmall_file", fs::remove_dir_all("f.txt"));
    show("nul", fs::write("a\0b", "x"));
    show("exists", fs::exists("d"));
    show("exists2", fs::exists("zz"));
    println!("{} {} {} {}", Path::new("d").exists(), Path::new("d").is_dir(), Path::new("f.txt").is_file(), Path::new("q").exists());
    show("rmdir", fs::remove_dir("d"));
    show("rmfile_dir", fs::remove_file("d"));
    match fs::read_dir("d") { Ok(rd) => { println!("{:?}", rd); for e in rd { match e { Ok(e) => println!("{:?} {:?} {:?} {:?}", e, e.file_name(), e.path(), e.file_type()), Err(e) => println!("{}", e) } } } Err(e) => println!("{}", e) }
    show("rd_file", fs::read_dir("f.txt"));
    show("rename", fs::rename("h.txt", "d/h2.txt"));
    show("read2", fs::read_to_string("d/h2.txt"));
    show("rename_missing", fs::rename("nope", "x"));
    match fs::canonicalize("d/e/../h2.txt") { Ok(p) => println!("canon {}", p.ends_with("d/h2.txt")), Err(e) => println!("{}", e) }
    show("canon_missing", fs::canonicalize("nope/x"));
    show("rmall", fs::remove_dir_all("d"));
    show("rmall2", fs::remove_dir_all("d"));
    println!("{}", Path::new("d").exists());
    for f in ["f.txt", "g.bin", "i.txt", "bad.txt"] { show("rm", fs::remove_file(f)); }
    show("rm2", fs::remove_file("f.txt"));
    show("cd back", std::env::set_current_dir(std::env::temp_dir()));
    show("cleanup", fs::remove_dir_all(&root));
    println!("{}", root.exists());
}
