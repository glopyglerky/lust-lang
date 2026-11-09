// A closure that calls through the enclosing fn's `F: Fn(.., &mut LinkBinders<'scope>)` bound,
// and passes `F` on to a recursive call (ts-rust program.rs spawn_bind_chain).
use std::sync::mpsc::Sender;

struct Queue { left: std::sync::atomic::AtomicUsize }
impl Queue {
    fn files_left(&self) -> bool { self.left.load(std::sync::atomic::Ordering::SeqCst) > 0 }
}

struct LinkBinders<'a> { queue: &'a Queue, count: usize }

fn spawn_bind_thread<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, f: impl FnOnce() + Send + 'scope) {
    std::thread::Builder::new().spawn_scoped(scope, f).unwrap();
}

fn spawn_bind_chain<'scope, F>(
    scope: &'scope std::thread::Scope<'scope, '_>,
    threads: usize,
    run: &'scope F,
    queue: &'scope Queue,
    sender: Sender<usize>,
) where
    F: Fn(Sender<usize>, &mut LinkBinders<'scope>) + Sync,
{
    spawn_bind_thread(scope, move || {
        let mut binders = LinkBinders { queue, count: threads };
        if threads > 1 && queue.files_left() {
            spawn_bind_chain(scope, threads - 1, run, queue, sender.clone());
        }
        binders.count = 1;
        run(sender, &mut binders);
        binders.count = 0;
    });
}

fn main() {
    let q = Queue { left: std::sync::atomic::AtomicUsize::new(3) };
    let (tx, rx) = std::sync::mpsc::channel();
    let run = |s: Sender<usize>, b: &mut LinkBinders<'_>| {
        b.queue.left.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        s.send(b.count).unwrap();
    };
    std::thread::scope(|scope| spawn_bind_chain(scope, 3, &run, &q, tx));
    let mut v: Vec<usize> = rx.iter().collect();
    v.sort();
    println!("{:?}", v);
}
