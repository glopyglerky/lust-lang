// Empty String and Vec values hold dangling pointers, so options
// over them niche-fill: Some(empty) and None must stay distinct.
fn main() {
    let some_empty: Option<String> = Some(String::new());
    let none: Option<String> = None;
    println!("{} {} {}", some_empty.is_some(), none.is_none(), some_empty.unwrap().len());

    let mut v: Vec<Option<String>> = Vec::new();
    v.push(None);
    v.push(Some(String::new()));
    v.push(Some(String::from("x")));
    println!("{} {} {}", v[0].is_none(), v[1].is_some(), v[2].is_some());
    match v.pop() {
        Some(Some(s)) => println!("pop {}", s),
        _ => println!("bad"),
    }
    match v.pop() {
        Some(Some(s)) => println!("pop empty {}", s.len()),
        _ => println!("bad"),
    }
    match v.pop() {
        Some(None) => println!("pop none"),
        _ => println!("bad"),
    }

    let some_vec: Option<Vec<u64>> = Some(Vec::new());
    println!("{} {}", some_vec.is_some(), some_vec.unwrap().len());

    let empty_str = String::new();
    let as_opt: Option<&str> = Some(empty_str.as_str());
    println!("{} {}", as_opt.is_some(), as_opt.unwrap().len());

    let ev: Vec<i32> = Vec::new();
    let first = ev.first();
    println!("{}", first.is_none());
    let one = vec![9i32];
    println!("{}", one.first().unwrap());

    // Options over slices from values that may be empty.
    let bytes = vec![65u8, 66];
    let slice_opt: Option<&[u8]> = Some(&bytes[..]);
    println!("{}", slice_opt.unwrap().len());
}
