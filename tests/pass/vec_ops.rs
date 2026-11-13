// Vec of structs, nested Vecs, vec![x; n], clone, equality, two-phase borrows.
#[derive(Debug, Clone, PartialEq)]
struct Cell {
    alive: bool,
    label: String,
}

fn main() {
    let mut grid = vec![vec![0i32; 4]; 3];
    grid[1][2] = 7;
    grid[2][0] = -1;
    println!("{:?}", grid);
    let mut v: Vec<i64> = Vec::new();
    v.push(1);
    v.push(v.len() as i64);
    v.push(v[0] + v[1]);
    println!("{:?} {}", v, v.is_empty());
    let cells = vec![Cell { alive: true, label: "a".to_string() }; 2];
    let mut copy = cells.clone();
    copy[1].label.push_str("b");
    println!("{:?}", copy);
    println!("{} {}", cells == copy, cells[0] == copy[0]);
    let words: Vec<&str> = vec!["x", "yy", "zzz"];
    let mut total = 0;
    for w in &words {
        total += w.len();
    }
    println!("{} {:?}", total, words);
    let empty: Vec<String> = Vec::with_capacity(10);
    println!("{:?} {}", empty, empty.len());
}
