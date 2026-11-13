fn usize_half(x: usize) -> u8 {
    match x {
        0.. => 1,
    }
}

fn isize_split(x: isize) -> u8 {
    match x {
        ..0 => 0,
        0.. => 1,
    }
}

fn slice_len(v: &[u8]) -> u8 {
    match v {
        [] => 0,
        [_] => 1,
        [_, _, ..] => 2,
    }
}

fn nested_slice(v: Option<&[u8]>) -> u8 {
    match v {
        None => 0,
        Some([]) => 1,
        Some([_]) => 2,
        Some([_, _, ..]) => 3,
    }
}

fn main() {
    // Literal and range patterns do auto-deref; const paths do not,
    // so the const form needs the explicit `&`.
    match &5i64 {
        5 => println!("lit"),
        _ => println!("no"),
    }
    match &5i64 {
        &i64::MIN => println!("min"),
        _ => println!("const"),
    }
    match &5u8 {
        &(0..=9) => println!("digit"),
        _ => println!("other"),
    }
    match [5u8] {
        [(1..)] => println!("paren"),
        _ => println!("other"),
    }
    let &mut m = &mut 7u8;
    let v: &[u8] = &[1, 2];
    let [..] = v;
    println!("{} {} {}", usize_half(9), isize_split(-3), isize_split(3));
    println!("{} {} {}", slice_len(&[]), slice_len(&[1]), slice_len(&[1, 2, 3]));
    println!("{} {}", nested_slice(Some(&[1])), nested_slice(None));
    println!("{}", m);
    'a: for i in 0..3 {
        'b: {
            if i == 1 {
                continue 'a;
            }
            println!("i{}", i);
        }
    }
}
