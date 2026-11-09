// let-else with return and panic else blocks, on Option and Result.
fn first(v: &Vec<String>) -> String {
    let Some(s) = v.first() else { return "empty".to_string(); };
    s.clone()
}
fn main() {
    let a: Option<i32> = Some(3);
    let Some(v) = a else { panic!("none"); };
    println!("{}", v);
    let b: Result<i32, String> = Err("bad".to_string());
    let Ok(n) = b else {
        println!("was err");
        return;
    };
    println!("unreachable {}", n);
}
fn unused() {
    let vs: Vec<String> = Vec::new();
    println!("{}", first(&vs));
}
