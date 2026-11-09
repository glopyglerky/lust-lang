fn classify(x: Option<i64>, y: Option<i64>) -> i64 {
    if let Some(a) = x && a > 0 && let Some(b) = y {
        a + b
    } else if let Some(a) = x {
        a
    } else {
        -1
    }
}

fn main() {
    println!("{}", classify(Some(3), Some(4)));
    println!("{}", classify(Some(3), None));
    println!("{}", classify(Some(-1), Some(4)));
    println!("{}", classify(None, Some(4)));

    let v = vec![10, 20, 30];
    let mut i = 0usize;
    let mut total = 0;
    while let Some(x) = v.get(i) && *x < 30 {
        total += x;
        i += 1;
    }
    println!("{} {}", total, i);

    let name = Some("lust".to_string());
    if let Some(s) = &name && s.len() == 4 && let Some(c) = s.chars().next() {
        println!("{} {}", s, c);
    }

    let p: Option<(i64, i64)> = Some((2, 3));
    if let Some((a, b)) = p && a + b == 5 {
        println!("sum");
    }
}
