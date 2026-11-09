// Range and RangeInclusive count, last, nth, min, and max are O(1),
// as are the delegating adapters (skip, take, enumerate, peekable,
// chain, rev nth): none of these walk the range.
fn main() {
    println!("{:?}", (0u64..u64::MAX).count());
    println!("{:?}", (0u64..=u64::MAX).last());
    println!("{:?}", (0u64..u64::MAX).nth(1 << 63));
    println!("{:?} {:?}", (0u64..u64::MAX).min(), (0u64..u64::MAX).max());
    println!("{} {:?}", (0u64..u64::MAX).skip(3).count(), (0u64..=u64::MAX).skip(2).last());
    println!("{:?}", (0u64..u64::MAX).skip(3).nth(1 << 63));
    println!("{:?}", (0u64..u64::MAX).step_by(1 << 62).collect::<Vec<_>>());
    println!("{:?} {:?}", (0u64..u64::MAX).rev().nth(1 << 63), (0u64..u64::MAX).take(usize::MAX).count());
    println!("{} {}", (0u64..u64::MAX).enumerate().count(), (0u64..u64::MAX).peekable().count());
    println!("{:?} {:?}", (i64::MIN..i64::MAX).nth(1 << 63), (-5i8..5).nth(7));
    // Small-range state stays exact after fast nth from both ends.
    let mut r = 250u8..=255;
    println!("{:?} {:?} {:?}", r.nth(5), r.next(), (0..=255u8).rev().nth(255));
    let mut r = 10..20;
    println!("{:?} {:?} {:?}", r.nth(3), r.nth(100), r.next());
    println!("{:?} {:?} {:?}", (0..20).step_by(3).nth(2), (0..20).step_by(usize::MAX).nth(1), (-10..10).step_by(4).nth(3));
    println!("{:?} {:?}", (0..3).chain(10..13).nth(4), (0..3).chain(10..10).last());
    // Back iteration stops at the element type's own MIN and MAX.
    println!("{:?}", (0u16..=8u16).rev().step_by(4).collect::<Vec<_>>());
    println!("{:?}", (-8i16..=0i16).step_by(4).collect::<Vec<_>>());
    let mut r = (0u16..=8u16).rev();
    println!("{:?} {:?} {:?} {:?}", r.nth(0), r.nth(3), r.nth(3), r.nth(0));
}
