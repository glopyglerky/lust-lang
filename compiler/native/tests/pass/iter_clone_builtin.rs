// clone() on builtin iterators std makes Clone, whose state is plain data
// (a clone is a bitwise copy): slice, str, and chunk iterators, the
// adapters over them with Copy closures, Peekable with a Copy item, and
// `I: Iterator + Clone` bounds at those types (ts-rust: CharIndices,
// Filter<slice::Iter, closure>).
fn twice<I: Iterator + Clone>(mut i: I) -> (usize, usize) {
    i.next();
    (i.clone().count(), i.count())
}

fn main() {
    let v = vec![1u8, 2, 3, 4, 5];
    let s = String::from("a,bc d\nef");
    let k = 10u8;

    let mut ci = s.char_indices();
    ci.next();
    let saved = ci.clone();
    println!("{:?} {:?}", ci.next(), saved.last());

    let big = v.iter().filter(|x| **x > 2);
    println!("{:?} {}", big.clone().collect::<Vec<_>>(), big.count());

    let mut p = v.iter().map(|x| x + k).peekable();
    p.peek();
    let q = p.clone();
    println!("{:?} {:?}", p.collect::<Vec<_>>(), q.count());

    println!("{:?} {:?} {:?}", twice(s.split(',')), twice(v.chunks(2).enumerate()), twice(s.chars().rev().skip(1)));
    println!("{:?} {:?}", twice(v.windows(3).zip(0..9)), twice((0..5).step_by(2)));
    let words = s.split_whitespace();
    for _ in 0..2 {
        println!("{}", words.clone().count());
    }
}
