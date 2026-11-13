// Std iterator and range types named through a `use` (TRIAGE
// T5): the import binds the same builtin Adt the full path
// names, so written types and values from methods agree.
use std::iter::Map;
use std::ops::{Range, RangeFrom, RangeInclusive};
use std::slice::Iter;
use std::str::Chars;

fn sum(r: Range<usize>) -> usize {
    let mut s = 0;
    for i in r {
        s += i;
    }
    s
}

fn first3(r: RangeFrom<usize>) -> usize {
    r.start
}

fn count_incl(r: RangeInclusive<u32>) -> usize {
    r.count()
}

fn total(it: Iter<'_, i32>) -> i32 {
    it.sum()
}

fn letters(c: Chars<'_>) -> usize {
    c.count()
}

fn doubled(m: Map<Range<usize>, fn(usize) -> usize>) -> usize {
    m.sum()
}

fn main() {
    println!("{}", sum(2..5));
    println!("{}", first3(10..));
    println!("{}", count_incl(3..=6u32));
    let v = vec![1i32, 2, 3];
    println!("{}", total(v.iter()));
    println!("{}", letters("héllo".chars()));
    println!("{}", doubled((1..4).map(|x| x * 2)));
    let r: Range<usize> = 4..4;
    println!("{} {}", r.is_empty(), (1..9).contains(&3));
}
