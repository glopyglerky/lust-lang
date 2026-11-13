// `std::thread::Result<T>`: std's alias for
// `Result<T, Box<dyn Any + Send>>` (TRIAGE R26; ts-rust's job results
// and `go_recover`), by full path, through `use std::thread`, in a
// type alias, and with `PoisonError::into_inner` as a fn value.
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

type JobResult<R> = std::thread::Result<R>;

fn run<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> std::thread::Result<R> {
    thread::spawn(f).join()
}

fn first(results: Vec<JobResult<u32>>) -> thread::Result<u32> {
    results.into_iter().next().unwrap_or_else(|| Err(Box::new(0u8)))
}

fn describe(r: &thread::Result<String>) -> String {
    match r {
        Ok(s) => s.clone(),
        Err(e) => e.downcast_ref::<String>().cloned().unwrap_or_default(),
    }
}

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let c = Arc::clone(&counter);
    let r: JobResult<()> = run(move || *c.lock().unwrap_or_else(PoisonError::into_inner) += 2);
    println!("{} {}", r.is_ok(), *counter.lock().unwrap_or_else(PoisonError::into_inner));
    println!("{:?}", first(vec![run(|| 9), run(|| 1)]).ok());
    println!("{}", first(Vec::new()).is_err());
    println!("{} {}", describe(&Ok("done".to_string())), describe(&Err(Box::new("lost".to_string()))));
}
