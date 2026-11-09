// `dyn` types with only auto traits (R25, ts-rust Box<dyn Send> garbage
// queues): coercions, drops through the vtable, containers, channels, and
// threads; and dyn sources that drop a principal or auto traits.
use std::sync::Mutex;
use std::sync::mpsc;
struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct PoolToken(Mutex<Vec<Box<dyn Send>>>);
fn drop_on_worker(value: Box<dyn Send>) {
    let (tx, rx) = mpsc::channel::<Box<dyn Send>>();
    tx.send(value).unwrap();
    let got = rx.recv().unwrap();
    println!("got one");
    drop(got);
}
enum Task {
    Drop(Box<dyn Send>),
    Nothing,
}
fn main() {
    drops();
    let a: Box<dyn Send> = Box::new(Noisy("a"));
    let b: Box<dyn Send + Sync> = Box::new(Noisy("b"));
    let c: Box<dyn Sync> = Box::new(5u64);
    let pool = PoolToken(Mutex::new(Vec::new()));
    pool.0.lock().unwrap().push(a);
    pool.0.lock().unwrap().push(Box::new(vec![Noisy("v1"), Noisy("v2")]));
    pool.0.lock().unwrap().push(Box::new(String::from("s")));
    println!("len {}", pool.0.lock().unwrap().len());
    drop_on_worker(Box::new(Noisy("w")));
    let t = Task::Drop(Box::new(Noisy("t")));
    if let Task::Drop(x) = t {
        println!("task");
        drop(x);
    }
    let _n = Task::Nothing;
    let opt: Option<Box<dyn Send>> = Some(Box::new(Noisy("o")));
    println!("{}", opt.is_some());
    drop(b);
    drop(c);
    println!("{}", std::mem::size_of::<Box<dyn Send>>());
    let h = std::thread::spawn(move || {
        let v = pool;
        println!("thread {}", v.0.lock().unwrap().len());
    });
    h.join().unwrap();
    println!("end");
}
trait Job: Send {
    fn run(&self) -> u8;
}
impl Job for Noisy {
    fn run(&self) -> u8 {
        self.0.len() as u8
    }
}
fn drops() {
    let j: Box<dyn Job> = Box::new(Noisy("job"));
    println!("run {}", j.run());
    let s: Box<dyn Send> = j;
    println!("dropped principal");
    drop(s);
    let both: Box<dyn Send + Sync> = Box::new(Noisy("both"));
    let one: Box<dyn Sync> = both;
    drop(one);
    let n = Noisy("ref");
    let r: &(dyn Send + Sync) = &n;
    let r2: &dyn Send = r;
    let _ = r2;
    let u: Box<dyn Unpin> = Box::new(Box::new(3u8) as Box<dyn Send>);
    drop(u);
}
