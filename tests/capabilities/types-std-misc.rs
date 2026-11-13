// capability: types: std::env::Args (from std::env::args())
// capability: types: str::Chars (from s.chars())
// capability: types: std::time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH} and core::time::Duration (std source modules; no layout queries)
// capability: types: std::marker::PhantomData<T> (std source; Hash, and Debug for a T whose type_name lust prints)
// capability: types: std::cmp::Reverse<T> with reversed Ord and PartialOrd, Debug, Hash, Default, Clone (std source)
// capability: types: std::ops::ControlFlow<B, C> with `?`, is_break/is_continue, break_value/continue_value, map_break/map_continue (std source)
// expect: ok

use std::cmp::Reverse;
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn find(v: &[u32]) -> ControlFlow<u32, ()> {
    for &x in v {
        if x > 2 {
            return ControlFlow::Break(x);
        }
    }
    ControlFlow::Continue(())
}

fn twice(v: &[u32]) -> ControlFlow<u32, u32> {
    find(v)?;
    ControlFlow::Continue(0)
}

fn main() {
    let args: std::env::Args = std::env::args();
    let mut chars: std::str::Chars = "ab".chars();
    let d = Duration::from_millis(1500) + core::time::Duration::from_secs(1);
    let start = Instant::now();
    let after = SystemTime::now().duration_since(UNIX_EPOCH).is_ok();
    let p: PhantomData<u32> = PhantomData;
    let mut v = vec![Reverse(1), Reverse(3), Reverse(2)];
    v.sort();
    let cf = twice(&[1, 5]);
    println!("{} {:?} {:?} {} {} {:?} {:?}", args.count() > 0, chars.next(), d, start.elapsed() < Duration::from_secs(60), after, p, v);
    println!("{} {:?} {:?}", cf.is_break(), cf.break_value(), find(&[1]).map_continue(|_| 0u8).continue_value());
}
