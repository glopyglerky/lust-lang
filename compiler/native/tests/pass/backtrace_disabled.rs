// std::backtrace: disabled(), status(), BacktraceStatus, and the Display and
// Debug of a disabled backtrace (tf-p6 R16; capture forms stay rejects).
use std::backtrace::{Backtrace, BacktraceStatus};
const B: Backtrace = Backtrace::disabled();
struct E { bt: Backtrace }
fn main() {
    let b = Backtrace::disabled();
    println!("[{}] [{:?}] [{:>30}] [{:<12?}] [{:#?}] [{:#}]", b, b, b, b, b, b);
    println!("{:?} {}", b.status(), b.status() == BacktraceStatus::Disabled);
    let s = b.to_string();
    println!("{} {}", s.len(), format!("{B}"));
    match B.status() { BacktraceStatus::Disabled => println!("d"), BacktraceStatus::Captured => println!("c"), _ => println!("other") }
    let e = E { bt: std::backtrace::Backtrace::disabled() };
    println!("{:?} {:?}", e.bt, std::backtrace::BacktraceStatus::Captured);
    let v = vec![Backtrace::disabled(), Backtrace::disabled()];
    println!("{:?}", v);
    let o: Option<Backtrace> = Some(Backtrace::disabled());
    println!("{:?}", o.map(|b| b.status()));
    fn sendsync<T: Send + Sync>(_: &T) {} sendsync(&b);
    let h = std::thread::spawn(move || b.to_string()); println!("{}", h.join().unwrap());
}
