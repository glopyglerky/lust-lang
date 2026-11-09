// expect: L0110
// rustc E0407: a user `Arg` impl cannot define the adapter hook.
struct P;

impl rustix::path::Arg for P {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

fn main() {
    println!("{}", rustix::fs::statvfs(P).is_ok());
}
