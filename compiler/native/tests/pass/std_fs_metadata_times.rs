// Metadata::modified and accessed (library/std/src/fs_time.rs): the stat
// times as SystemTime, through `?`, Option, UFCS and a closure.
use std::time::{SystemTime, UNIX_EPOCH};

fn times(p: &str) -> std::io::Result<(SystemTime, SystemTime)> {
    let md = std::fs::metadata(p)?;
    Ok((md.modified()?, md.accessed()?))
}

fn main() {
    let (m, a) = times("/").unwrap();
    println!("{} {}", m > UNIX_EPOCH, a > UNIX_EPOCH);
    let md = std::fs::symlink_metadata("/").unwrap();
    println!("{}", std::fs::Metadata::modified(&md).unwrap() == m);
    let f = |md: &std::fs::Metadata| md.modified().ok();
    println!("{:?}", f(&md).map(|t| t.duration_since(UNIX_EPOCH).is_ok()));
    match times("/no/such/file") {
        Ok(_) => println!("found"),
        Err(e) => println!("{:?}", e.kind()),
    }
}
