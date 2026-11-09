// `io::read` over upstream's three counting buffer forms.
use std::io::Write;

fn main() {
    let (r, w) = rustix::pipe::pipe().unwrap();
    let mut f = std::fs::File::from(w);
    f.write_all(b"abcdefgh").unwrap();
    let mut a = [0u8; 3];
    let n1 = rustix::io::read(&r, &mut a).unwrap();
    let mut v = vec![0u8; 2];
    let n2 = rustix::io::read(&r, &mut v).unwrap();
    let mut s = [0u8; 2];
    let n3 = rustix::io::read(&r, &mut s[..]).unwrap();
    println!("{n1} {:?} {n2} {:?} {n3} {:?}", a, v, s);
    println!("{}", rustix::fs::Mode::RUSR.bits() | rustix::fs::Mode::WUSR.bits());
    println!("{}", rustix::io::Errno::ACCESS.raw_os_error());
}
