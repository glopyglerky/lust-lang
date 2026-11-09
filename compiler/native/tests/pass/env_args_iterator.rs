// std::env::Args is an Iterator, DoubleEndedIterator, and
// ExactSizeIterator: every adapter runs on it. The program name
// (argument 0) differs between builds, so only its presence prints.
use std::env;

fn run(args: impl Iterator<Item = String>) -> usize {
    args.count()
}

fn take_all(a: env::Args) -> Vec<String> {
    a.collect()
}

fn exact<I: ExactSizeIterator>(i: I) -> usize {
    i.len()
}

fn main() {
    let rest: Vec<String> = env::args().skip(1).collect();
    println!("{:?}", rest);
    println!("{} {}", env::args().len(), exact(env::args()));
    for (i, s) in env::args().enumerate() {
        println!("{} {}", i, !s.is_empty());
    }
    let mut it = env::args();
    println!("{:?} {:?}", it.size_hint(), it.nth(3));
    println!("{:?} {}", it.next_back(), it.len());
    println!("{}", run(env::args().skip(1)));
    println!("{}", take_all(env::args()).len());
    let back: Vec<usize> = env::args().rev().map(|s| s.len().min(1)).collect();
    println!("{:?}", back);
    println!("{:?}", env::args().last().map(|s| s.is_empty()));
    println!("{}", env::args().count());
    let mut p = env::args().peekable();
    while let Some(x) = p.next() {
        println!("{} {:?}", x.is_empty(), p.peek().map(|s| s.len()));
    }
    let a: std::env::Args = env::args();
    println!("{}", a.filter(|s| s.starts_with('-')).count());
}
