// std::fs::File and OpenOptions (W-SHIM phase 2): open/create,
// access modes and their errors, Read/Write/Seek through File,
// set_len, sync, metadata, try_clone, and Debug forms.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

fn show<T: std::fmt::Debug>(label: &str, r: io::Result<T>) {
    match r {
        Ok(v) => println!("{label} Ok({v:?})"),
        Err(e) => println!("{label} Err({e} {:?})", e.kind()),
    }
}

fn main() {
    fs::write("a.txt", "hello world").unwrap();
    let mut f = File::open("a.txt").unwrap();
    let mut b = [0u8; 5];
    show("read", f.read(&mut b));
    println!("{b:?}");
    show("write-ro", f.write_all(b"x"));
    match f.metadata() {
        Ok(m) => println!("meta-len Ok({})", m.len()),
        Err(e) => println!("meta-len Err({e} {:?})", e.kind()),
    }
    show("tell", f.stream_position());
    show("seek-start", f.seek(SeekFrom::Start(6)));
    let mut s = String::new();
    show("rest", f.read_to_string(&mut s));
    println!("{s:?}");
    show("seek-end-neg", f.seek(SeekFrom::End(-5)));
    show("seek-cur", f.seek(SeekFrom::Current(-2)));
    show("rewind", f.rewind());
    let mut v: Vec<u8> = Vec::new();
    show("to-end", f.read_to_end(&mut v));
    println!("{v:?}");
    // create truncates; write; set_len grows and shrinks.
    let mut g = File::create("b.txt").unwrap();
    show("read-wo", g.read(&mut b));
    show("write", g.write_all(b"abcdef"));
    show("sync", g.sync_all());
    show("sync-data", g.sync_data());
    show("set-len-3", g.set_len(3));
    show("set-len-5", g.set_len(5));
    println!("{:?}", fs::read("b.txt").unwrap());
    let clone = g.try_clone();
    match clone {
        Ok(mut c) => {
            show("clone-seek", c.seek(SeekFrom::Start(1)));
            show("g-tell", g.stream_position());
        }
        Err(e) => println!("clone err {e}"),
    }
    drop(g);
    // append.
    let mut a = OpenOptions::new().append(true).open("b.txt").unwrap();
    show("append-write", a.write_all(b"Z"));
    drop(a);
    println!("{:?}", fs::read("b.txt").unwrap());
    // read/write with seek and overwrite.
    let mut rw = OpenOptions::new().read(true).write(true).open("a.txt").unwrap();
    show("rw-seek", rw.seek(SeekFrom::Start(5)));
    show("rw-write", rw.write_all(b"-"));
    println!("{:?}", fs::read_to_string("a.txt").unwrap());
    // Errors.
    show("open-missing", File::open("nope.txt"));
    show("create-new-exists", OpenOptions::new().write(true).create_new(true).open("a.txt"));
    // (not through `show`: File's Debug prints its absolute path,
    // which is the run directory and differs between the two runs)
    match OpenOptions::new().write(true).create_new(true).open("c.txt") {
        Ok(_) => println!("create-new-ok Ok"),
        Err(e) => println!("create-new-ok Err({e} {:?})", e.kind()),
    }
    show("no-flags", OpenOptions::new().open("a.txt"));
    show("read-truncate", OpenOptions::new().read(true).truncate(true).open("a.txt"));
    show("read-create", OpenOptions::new().read(true).create(true).open("a.txt"));
    show("append-truncate", OpenOptions::new().append(true).truncate(true).open("a.txt"));
    show("nul", File::open("a\0.txt"));
    show("seek-before", f.seek(SeekFrom::Current(-100)));
    // File as a process stdio target through Stdio::from.
    let target = File::create("out.txt").unwrap();
    let status = std::process::Command::new("printf").arg("to-file").stdout(std::process::Stdio::from(target)).status().unwrap();
    println!("{status}");
    println!("{:?}", fs::read("out.txt").unwrap());
    // Debug: File shows fd, absolute path, access (path tail compared
    // loosely by only printing flags around it).
    println!("{:?}", OpenOptions::new().read(true).append(true));
    let f2 = File::open("a.txt").unwrap();
    let text = format!("{f2:?}");
    println!("file-debug-has a.txt={} read={}", text.contains("a.txt"), text.contains("read: true"));
    for p in ["a.txt", "b.txt", "c.txt", "out.txt"] {
        let _ = fs::remove_file(p);
    }
}
