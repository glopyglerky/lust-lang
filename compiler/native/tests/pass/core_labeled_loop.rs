fn main() {
    let mut n = 0;
    'outer: loop {
        n += 1;
        if n > 1 {
            break 'outer;
        }
    }
    println!("{}", n);
}
