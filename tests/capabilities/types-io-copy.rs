// capability: types: std::io::copy on std's stack-buffer path (not for &[u8], VecDeque, BufReader readers, Vec<u8> or BufWriter writers, or an OS handle on both sides)
// expect: ok

use std::io::Write;

struct Src(u32);

impl std::io::Read for Src {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.0 == 0 {
            return Ok(0);
        }
        self.0 -= 1;
        buf[0] = b'a';
        Ok(1)
    }
}

struct Count(usize);

impl Write for Count {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let mut w = Count(0);
    let n = std::io::copy(&mut Src(3), &mut w).unwrap();
    println!("{} {}", n, w.0);
}
