// capability: control_flow: if / else
// capability: control_flow: if let and let chains
// capability: control_flow: let-else
// capability: control_flow: match (guards, or patterns, ranges, nested patterns)
// capability: control_flow: loop (break with a value)
// capability: control_flow: while
// capability: control_flow: while let
// capability: control_flow: for over any IntoIterator (ranges, collections, iterator chains, s.chars())
// capability: control_flow: break and continue, with labels
// capability: control_flow: labeled blocks with break values
// capability: control_flow: early return
// capability: control_flow: blocks as expressions
// expect: ok

fn classify(n: i32) -> &'static str {
    if n < 0 {
        return "neg";
    }
    match n {
        0 => "zero",
        1 | 2 => "small",
        3..=9 if n % 2 == 1 => "odd digit",
        3..=9 => "even digit",
        _ => "big",
    }
}

fn head(v: &[u32]) -> u32 {
    let Some(&first) = v.first() else {
        return 0;
    };
    first
}

fn main() {
    let o = Some(Some(4));
    if let Some(Some(x)) = o
        && x > 3
    {
        println!("chain {x}");
    } else {
        println!("no");
    }
    let mut i = 0;
    let found = loop {
        i += 1;
        if i == 5 {
            break i * 10;
        }
    };
    while i > 3 {
        i -= 1;
    }
    let mut stack = vec![1, 2];
    while let Some(t) = stack.pop() {
        i += t;
    }
    let mut total = 0;
    'outer: for a in 0..4 {
        for b in [1, 2, 3].iter().map(|x| x * 2) {
            if b == 4 {
                continue 'outer;
            }
            if a == 3 {
                break 'outer;
            }
            total += a + b;
        }
    }
    for c in "ab".chars() {
        total += c as i32;
    }
    let lb = 'blk: {
        if total > 0 {
            break 'blk 1;
        }
        2
    };
    let blk = {
        let t = 3;
        t * 2
    };
    println!("{} {} {} {} {} {} {} {}", classify(-1), classify(7), classify(4), found, i, total, lb + blk, head(&[]));
}
