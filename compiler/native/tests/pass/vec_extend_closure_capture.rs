// `Vec::extend` takes the items' regions, not a map closure's captures (R18).
fn r1(areas: &[Vec<String>]) -> usize {
    let mut e: Vec<(&str, usize)> = Vec::new();
    for (task, area) in areas.iter().enumerate() { e.extend(area.iter().map(|k| (k.as_str(), task))); }
    e.len()
}
fn r3(areas: &[Vec<String>]) -> usize {
    let mut e: Vec<&str> = Vec::new();
    for (task, area) in areas.iter().enumerate() { e.extend(area.iter().filter(|_| task > 0).map(|k| k.as_str())); }
    e.len()
}
fn main() { let a = vec![vec!["a".to_string()], vec!["b".to_string()]]; println!("{} {}", r1(&a), r3(&a)); }
