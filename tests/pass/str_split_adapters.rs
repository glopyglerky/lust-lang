// Split iterators under adapters, for loops, and split_once (W-STD-B).
fn main() {
    let csv = "name, age ,city\nbob,4,  rome\n";
    for (i, line) in csv.lines().enumerate() {
        let cols: Vec<String> = line.split(',').map(|c| c.trim().to_string()).collect();
        println!("{i} {cols:?}");
    }
    let mut it = "a b c d".split(' ');
    let n = it.nth(1);
    println!("{:?} {:?}", n, it.last());
    let s = "1-2-3";
    let total: usize = s.split('-').map(|x| x.len()).sum();
    println!("{total} {:?}", s.split('-').size_hint());
    let pairs: Vec<(&str, &str)> = "a=1;b=2".split(';').filter_map(|kv| kv.split_once('=')).collect();
    println!("{pairs:?}");
    println!("{:?}", "x,y".split(',').zip("1 2".split(' ')).collect::<Vec<_>>());
    println!("{:?}", "a,b,c".rsplit(',').skip(1).collect::<Vec<_>>());
}
