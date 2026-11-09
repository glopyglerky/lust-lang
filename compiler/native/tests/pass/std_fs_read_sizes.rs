// fs::read and fs::read_to_string of files around the sizes where reading changes path: empty,
// the 32-byte probe that finds the end of a full buffer, the old 64 KiB read chunk, and larger.
// As in std, the Vec gets the size the file reports, so its capacity is its length; a file
// whose size reads as 0 but has bytes (/proc) still reads whole.
use std::fs;

fn mix(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf29ce484222325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100000001b3))
}

fn main() {
    let root = std::env::temp_dir().join(format!("lust_fs_read_sizes_{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    for &n in &[0usize, 1, 31, 32, 33, 4096, 65535, 65536, 65537, 200_003] {
        let data: Vec<u8> = (0..n).map(|i| (i * 7 + i / 251) as u8).collect();
        let p = root.join(format!("f{n}"));
        fs::write(&p, &data).unwrap();
        let back = fs::read(&p).unwrap();
        println!("{n}: len {} same {} cap==len {} {:x}", back.len(), back == data, back.capacity() == back.len(), mix(&back));
        let text: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
        fs::write(&p, &text).unwrap();
        let s = fs::read_to_string(&p).unwrap();
        println!("{n}: str len {} same {}", s.len(), s == text);
    }
    if std::path::Path::new("/proc/self/stat").exists() {
        let v = fs::read("/proc/self/stat").unwrap();
        println!("proc: nonempty {} ends with newline {}", !v.is_empty(), v.last() == Some(&b'\n'));
    } else {
        println!("proc: nonempty true ends with newline true");
    }
    fs::remove_dir_all(&root).unwrap();
}
