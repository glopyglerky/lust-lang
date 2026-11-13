use std::rc::Rc;
fn main() {
    let s = String::from("stop");
    match &*s {
        "go" => println!("go"),
        "stop" => println!("stop"),
        _ => println!("other"),
    }
    let t: &str = &*s;
    println!("{}", t);
    let mut v = vec![1, 2, 3];
    match &*v {
        [1, ..] => println!("one"),
        _ => println!("other"),
    }
    let sl: &[i32] = &*v;
    println!("{}", sl.len());
    if let [9, ..] = &mut *v {
        println!("nine");
    } else if let [first, ..] = &mut *v {
        *first = 9;
    }
    println!("{:?}", v);
    let rc = Rc::new(String::from("deep"));
    match &**rc {
        "deep" => println!("deep"),
        _ => println!("other"),
    }
    let mut w = String::from("ab");
    for c in (*w).chars() {
        print!("{}", c);
    }
    println!();
    let a = [4, 5, 6];
    println!("{}", a.as_slice().len());
    let mut b = [4, 5, 6];
    b.as_mut_slice()[0] = 40;
    println!("{:?}", b);
}
