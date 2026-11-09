// `collect` into `Result<Vec<_>, E>` and `Option<Vec<_>>` (F3):
// short-circuit at the first failure, std's capacity on the
// success path, and std's drop order on the failure path (the
// source iterator and its unconsumed items drop before the
// built prefix).
use std::cell::Cell;

thread_local! {
    static LOG: Cell<u64> = const { Cell::new(0) };
}
fn log(id: u64) { LOG.with(|l| l.set(l.get() * 100 + id)); }

struct D(u64);
impl Drop for D { fn drop(&mut self) { log(self.0); } }

fn main() {
    let ok: Result<Vec<i32>, &str> = vec![Ok(1), Ok(2), Ok(3)].into_iter().collect();
    println!("{:?}", ok);
    let err: Result<Vec<i32>, &str> = vec![Ok(1), Err("x"), Ok(3)].into_iter().collect();
    println!("{:?}", err);
    let first: Result<Vec<i32>, &str> = vec![Err("a"), Err("b")].into_iter().collect();
    println!("{:?}", first);
    let some: Option<Vec<i32>> = vec![Some(1), Some(2)].into_iter().collect();
    println!("{:?}", some);
    let none: Option<Vec<i32>> = vec![Some(1), None, Some(3)].into_iter().collect();
    println!("{:?}", none);
    // Pulls stop at the first Err.
    let pulls = Cell::new(0);
    let it = (0..10).map(|i| { pulls.set(pulls.get() + 1); if i == 2 { Err(()) } else { Ok(i) } });
    let r: Result<Vec<i32>, ()> = it.collect();
    println!("pulls {} {:?}", pulls.get(), r);
    // Capacities: the generic build over the short-circuit
    // wrapper, with in-place reuse of a Vec source's buffer.
    let v: Vec<Result<i32, ()>> = (0..5).map(Ok).collect();
    let r: Result<Vec<i32>, ()> = v.into_iter().collect();
    println!("cap reuse {}", r.unwrap().capacity());
    let r2: Result<Vec<i32>, ()> = (0..5).map(Ok).collect();
    println!("cap range {}", r2.unwrap().capacity());
    let o2: Option<Vec<i32>> = vec![Some(1), Some(2)].into_iter().collect();
    println!("cap opt {}", o2.unwrap().capacity());
    // Drop order on Err: unconsumed 4, 5 (source order), then
    // collected 1, 2, then the marker, then the error 3.
    LOG.with(|l| l.set(0));
    {
        let v: Vec<Result<D, D>> = vec![Ok(D(1)), Ok(D(2)), Err(D(3)), Ok(D(4)), Ok(D(5))];
        let r: Result<Vec<D>, D> = v.into_iter().collect();
        match r { Ok(_) => println!("unexpected ok"), Err(e) => { log(90); drop(e); } }
    }
    println!("drops {}", LOG.with(|l| l.get()));
    // Success keeps every value (and drops them front to back).
    LOG.with(|l| l.set(0));
    {
        let r: Option<Vec<D>> = vec![Some(D(1)), Some(D(2))].into_iter().collect();
        println!("kept {}", r.is_some());
    }
    println!("drops ok {}", LOG.with(|l| l.get()));
}
