fn main() {
    let mut count = 0;
    'outer: loop {
        let mut i = 0;
        loop {
            i += 1;
            if i == 2 {
                break;
            }
            count += 1;
        }
        if count > 3 {
            break 'outer;
        }
        count += 10;
    }
    println!("{}", count);

    let mut n = 0;
    'skip: loop {
        n += 1;
        if n < 3 {
            continue 'skip;
        }
        break 'skip;
    }
    println!("{}", n);

    let v = 'blk: {
        if n > 0 {
            break 'blk 42;
        }
        0
    };
    println!("{}", v);

    let w: i64 = {
        let mut i = 0;
        'again: loop {
            i += 1;
            if i < 5 {
                continue 'again;
            }
            break 'again i * 2;
        }
    };
    println!("{}", w);

    let mut t = 0;
    'rows: for r in 0..3 {
        for c in 0..3 {
            if r == c {
                continue 'rows;
            }
            t += 1;
        }
        t += 100;
    }
    println!("{}", t);

    let mut u = 0;
    'wl: while u < 10 {
        u += 1;
        if u == 4 {
            break 'wl;
        }
    }
    println!("{}", u);
}
