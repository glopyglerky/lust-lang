// Iterator adapters over Vec, slice, array, and range sources
// (W-STD-A phase 3): each adapter's items in order.
fn main() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let a = [10u8, 20, 30];
    let k = 3;
    println!("{:?}", v.iter().map(|x| x * k).collect::<Vec<_>>());
    println!("{:?}", v.iter().filter(|x| **x % 2 == 0).collect::<Vec<_>>());
    for (i, x) in a.iter().enumerate() {
        print!("{}={} ", i, x);
    }
    println!();
    println!("{:?}", v.iter().rev().collect::<Vec<_>>());
    println!("{:?}", v.iter().skip(2).take(3).collect::<Vec<_>>());
    println!("{:?}", v.iter().copied().chain(a.iter().map(|x| *x as i32)).collect::<Vec<i32>>());
    println!("{:?}", v.iter().zip(a.iter()).collect::<Vec<_>>());
    println!("{:?}", v.iter().copied().filter_map(|x| if x > 3 { Some(x * 100) } else { None }).collect::<Vec<i32>>());
    println!("{:?}", a.iter().cloned().collect::<Vec<u8>>());
    println!("{:?}", v.iter().take_while(|x| **x < 4).collect::<Vec<_>>());
    println!("{:?}", v.iter().skip_while(|x| **x < 4).collect::<Vec<_>>());
    println!("{:?}", v.iter().step_by(4).collect::<Vec<_>>());
    println!("{:?}", (0..20).step_by(7).collect::<Vec<i32>>());
    let mut seen = Vec::new();
    let total: i32 = v.iter().inspect(|x| seen.push(**x)).sum();
    println!("{} {:?}", total, seen);
    let s = &v[1..4];
    println!("{:?}", s.iter().map(|x| x + 1).rev().collect::<Vec<i32>>());
    let mut w = vec![1, 2, 3];
    for x in w.iter_mut() {
        *x *= 7;
    }
    w.iter_mut().for_each(|x| *x += 1);
    println!("{:?}", w);
    let names = vec![String::from("ann"), String::from("bo")];
    let lens: Vec<usize> = names.iter().map(|s| s.len()).collect();
    println!("{:?}", lens);
    let owned: Vec<String> = names.into_iter().rev().collect();
    println!("{:?}", owned);
    println!("{:?}", (1..=4).rev().chain(7..9).collect::<Vec<u64>>());
    let mut p = v.iter().peekable();
    while let Some(x) = p.next() {
        if let Some(n) = p.peek() {
            print!("{}<{} ", x, n);
        }
    }
    println!();
}
