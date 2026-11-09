// ts-rust pprof CpuProfiler (TRIAGE R2): Mutex<Option<ManuallyDrop<_>>>, a
// field read through Deref, and into_inner to drop the session.
use std::mem::ManuallyDrop;
use std::sync::Mutex;

// No heap fields: the last session is never stopped, so it leaks (as in
// ts-rust), and a leaked heap buffer would fail SANITIZE.
struct Session {
    path: &'static str,
    file: Option<usize>,
}
impl Drop for Session {
    fn drop(&mut self) {
        println!("stop {} {:?}", self.path, self.file);
    }
}

#[derive(Default)]
struct Profiler {
    mu: Mutex<Option<ManuallyDrop<Session>>>,
}

impl Profiler {
    fn start(&self, p: &'static str) -> Result<(), String> {
        let mut s = self.mu.lock().unwrap();
        if s.is_some() {
            return Err("already".to_string());
        }
        *s = Some(ManuallyDrop::new(Session { path: p, file: Some(p.len()) }));
        Ok(())
    }
    fn stop(&self) -> Result<String, String> {
        let mut s = self.mu.lock().unwrap();
        let Some(stopped) = s.take() else {
            return Err("not running".to_string());
        };
        let path = stopped.path.to_string();
        drop(ManuallyDrop::into_inner(stopped));
        Ok(path)
    }
}

fn main() {
    let p = Profiler::default();
    println!("{:?}", p.start("a"));
    println!("{:?}", p.start("b"));
    println!("{:?}", p.stop());
    println!("{:?}", p.stop());
    println!("{:?}", p.start("c"));
    drop(p);
    println!("end");
}
