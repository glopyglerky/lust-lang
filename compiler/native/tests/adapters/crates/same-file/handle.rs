// same-file 1.0.6: Handle construction, equality, hashing,
// dev and ino, and reads through as_file. Identity values are
// compared, never printed raw.
use same_file::Handle;
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    // Per process: the oracle and lust runs, or two gates on one box, must not share it.
    let dir = std::env::temp_dir().join(format!("lust_same_file_handle_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, "handle contents").unwrap();
    std::fs::write(&b, "handle contents").unwrap();

    let ha = Handle::from_path(&a).unwrap();
    let ha2 = Handle::from_path(&a).unwrap();
    let hb = Handle::from_path(&b).unwrap();
    println!("{} {}", ha == ha2, ha == hb);
    println!("{} {}", ha.dev() == hb.dev(), ha.ino() == hb.ino());
    println!("{}", ha.dev() > 0 && ha.ino() > 0);
    let mut set = HashSet::new();
    set.insert(Handle::from_path(&a).unwrap());
    println!("{} {}", set.contains(&ha2), set.contains(&hb));

    let f = std::fs::File::open(&a).unwrap();
    let mut hf = Handle::from_file(f).unwrap();
    println!("{}", hf == ha);
    let mut text = String::new();
    hf.as_file_mut().read_to_string(&mut text).unwrap();
    println!("{}", text);
    hf.as_file_mut().seek(SeekFrom::Start(7)).unwrap();
    let mut rest = String::new();
    hf.as_file().read_to_string(&mut rest).unwrap();
    println!("{}", rest);

    let e = Handle::from_path(dir.join("nope.txt")).unwrap_err();
    println!("{:?} {}", e.kind(), e);
    let _ = std::fs::remove_dir_all(&dir);
}
