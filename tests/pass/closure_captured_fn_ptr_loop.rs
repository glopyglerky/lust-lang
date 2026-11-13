// A call through a captured fn pointer in a loop of a closure (ts-rust program.rs
// start_stealing_group_do): the copy of the pointer is a temporary, assigned on each pass.
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Clone, Copy)]
struct Node(u32);
struct Checker { n: u32 }
struct StealQueues { next: AtomicUsize }
impl StealQueues {
    fn next(&self, _me: usize, files: &[Node]) -> Option<usize> {
        let i = self.next.fetch_add(1, Ordering::SeqCst);
        (i < files.len()).then_some(i)
    }
}
fn send_job<R>(_me: usize, f: impl FnOnce(&mut Checker) -> R) -> R {
    let mut c = Checker { n: 1 };
    f(&mut c)
}
fn group(files: Arc<Vec<Node>>, cb: fn(&mut Checker, Node) -> Vec<u32>, queues: Arc<StealQueues>) -> Vec<Vec<(usize, Vec<u32>)>> {
    (0..2)
        .map(|me| {
            let files = Arc::clone(&files);
            let queues = Arc::clone(&queues);
            send_job(me, move |checker| {
                let mut results = Vec::new();
                while let Some(i) = queues.next(me, &files) {
                    results.push((i, cb(checker, files[i])));
                }
                results
            })
        })
        .collect()
}
fn d(c: &mut Checker, n: Node) -> Vec<u32> { c.n += 1; vec![n.0, c.n] }
fn main() {
    let r = group(Arc::new(vec![Node(5), Node(6), Node(7)]), d, Arc::new(StealQueues { next: AtomicUsize::new(0) }));
    println!("{:?}", r);
}
