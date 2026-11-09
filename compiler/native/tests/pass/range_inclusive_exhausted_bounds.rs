// RangeInclusive after its last item: next leaves `exhausted` as the step overflow, a
// try_fold pull (any/find/position/all on &mut) sets it; end_bound and contains follow it.
use std::ops::RangeBounds;
fn st(r: &std::ops::RangeInclusive<i32>) -> String {
    format!("{} {} {} {:?} {}", r.start(), r.end(), r.is_empty(), r.end_bound(), r.contains(&3))
}
fn main() {
    let mut r = 1..=3; let a = (&mut r).any(|x| x > 5); println!("{a} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).any(|x| x == 3); println!("{a} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).any(|x| x == 2); println!("{a} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).position(|x| x == 9); println!("{a:?} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).nth(2); println!("{a:?} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).find(|&x| x == 3); println!("{a:?} {}", st(&r));
    let mut r = 1..=3; let a = (&mut r).all(|x| x < 9); println!("{a:?} {}", st(&r));
    let mut r = 3..=3; r.next(); println!("{}", st(&r));
    let mut r = 3..=3; r.next_back(); println!("{}", st(&r));
    let mut r = 255u8..=255; r.next(); println!("{} {:?} {}", r.contains(&255), r.end_bound(), r.is_empty());
    let mut r = 0u8..=0; r.next_back(); println!("{} {:?} {:?}", r.contains(&0), r.end_bound(), r.end());
    let mut r = 250u8..=255; for _ in 0..6 { r.next(); } println!("{} {}", r.contains(&5), r.contains(&255));
}
