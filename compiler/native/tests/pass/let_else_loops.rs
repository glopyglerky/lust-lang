// let-else with break and continue, irrefutable tuple pattern, by-value scrutinee.
fn main() {
    let mut i = 0;
    loop {
        let x: Option<i32> = if i < 3 { Some(i) } else { Option::None };
        let Some(v) = x else { break; };
        println!("{}", v);
        i += 1;
        let y: Option<i32> = if i == 2 { Option::None } else { Some(i * 10) };
        let Some(w) = y else { continue; };
        println!("w{}", w);
    }
    let s = "hi".to_string();
    let Ok(t) = Ok::<String, String>(s) else { return; };
    println!("{}", t);
    let p = (1, "two".to_string());
    let (n, word) = p else { return; };
    println!("{} {}", n, word);
}
