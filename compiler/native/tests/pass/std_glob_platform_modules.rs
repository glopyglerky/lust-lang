// A glob of a std platform module (io, fs, path, env, process) binds that
// module's items, and io's own `Result<T>` shadows the prelude's. Globs
// of std::fmt and std::thread bring their own `Result` too.
use std::io::*;
mod paths {
    use std::path::*;
    pub fn parent(s: &str) -> Option<PathBuf> {
        Path::new(s).parent().map(|p| p.to_path_buf())
    }
}
mod show {
    use std::fmt::*;
    pub struct S(pub u8);
    impl Display for S {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "S{}", self.0)
        }
    }
}
fn io_part() -> Result<usize> {
    let mut out = stdout();
    out.write_all(b"io glob\n")?;
    let e = Error::new(ErrorKind::Other, "boom");
    println!("{} {:?}", e, e.kind());
    let mut s = String::new();
    let n = BufReader::new(&b"line one\nline two\n"[..]).read_line(&mut s)?;
    print!("{s}");
    Ok(n)
}
fn main() {
    println!("{:?}", io_part());
    let r: std::result::Result<u8, u8> = Ok(3);
    println!("{:?} {:?}", r, paths::parent("a/b/c"));
    println!("{}", show::S(4));
    {
        use std::fs::*;
        println!("{}", read_to_string("/no/such/file/lust").is_err());
    }
    {
        use std::env::*;
        println!("{}", var("LUST_NO_SUCH_VAR_X").is_err());
    }
    {
        use std::thread::*;
        // The block's thread glob shadows the module's io glob.
        let j: Result<u8> = spawn(|| 9).join();
        println!("{:?}", j.is_ok());
    }
}
