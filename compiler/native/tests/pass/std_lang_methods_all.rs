// Every std source method of Option and Result (library/core), each arm.
#[derive(Debug, Clone, Default, PartialEq)]
struct P { x: i32 }

fn half(x: i32) -> Option<i32> { if x % 2 == 0 { Some(x / 2) } else { None } }
fn parse(s: &str) -> Result<i32, String> { if s == "bad" { Err(format!("bad input {}", s)) } else { Ok(s.len() as i32) } }

fn main() {
    let a: Option<i32> = Some(4);
    let n: Option<i32> = None;
    println!("{} {} {} {}", a.is_some(), a.is_none(), n.is_some(), n.is_none());
    println!("{} {}", a.is_some_and(|x| x > 3), n.is_none_or(|x| x > 100));
    println!("{:?} {:?}", a.map(|x| x + 1), n.map(|x| x + 1));
    println!("{} {}", a.map_or(0, |x| x * 3), n.map_or(7, |x| x * 3));
    println!("{} {}", a.map_or_else(|| -1, |x| x * 3), n.map_or_else(|| -1, |x| x * 3));
    println!("{} {}", a.map_or_default(|x| x * 3), n.map_or_default(|x| x * 3));
    println!("{:?} {:?}", a.and_then(|x| half(x)), n.and_then(|x| half(x)));
    println!("{:?} {:?}", a.and_then(|x| half(x)).and_then(|x| half(x)).and_then(|x| half(x)), Some(3).and_then(|x| half(x)));
    println!("{:?} {:?}", a.filter(|x| *x > 10), a.filter(|x| *x < 10));
    println!("{:?} {:?} {:?}", a.or(Some(9)), n.or(Some(9)), n.or_else(|| Some(11)));
    println!("{:?} {:?} {:?}", a.xor(Some(1)), a.xor(None), n.xor(Some(2)));
    println!("{:?} {:?}", a.zip(Some("s")), a.zip(None::<u8>));
    println!("{:?}", Some((1, 'c')).unzip());
    println!("{:?} {:?}", a.ok_or("e"), n.ok_or_else(|| format!("none{}", 1)));
    println!("{:?} {:?}", a.and(Some('z')), n.and(Some('z')));
    println!("{} {} {}", a.unwrap_or(0), n.unwrap_or(0), n.unwrap_or_else(|| 42));
    println!("{} {}", n.unwrap_or_default(), Some(String::from("q")).unwrap_or_default());
    let s: Option<String> = Some("hello".to_string());
    println!("{:?} {:?}", s.as_ref().map(|t| t.len()), s.as_ref().cloned());
    let mut m = Some(P { x: 1 });
    if let Some(p) = m.as_mut() { p.x += 10; }
    println!("{:?}", m);
    let v = vec![1, 2, 3];
    println!("{:?} {:?}", v.first().copied(), v.get(9).copied());
    let mut w = vec![5];
    println!("{:?}", w.first_mut().copied());
    println!("{:?}", w.last_mut().cloned());
    let mut t = Some(3);
    println!("{:?} {:?}", t.take(), t);
    println!("{:?} {:?}", t.replace(8), t);
    println!("{:?} {:?}", t.take_if(|x| *x > 100), t);
    println!("{:?} {:?}", t.take_if(|x| { *x += 1; *x > 5 }), t);
    let mut g: Option<i32> = None;
    *g.get_or_insert(1) += 1;
    *g.get_or_insert_with(|| 100) += 1;
    println!("{:?}", g);
    let mut d: Option<P> = None;
    d.get_or_insert_default().x = 5;
    println!("{:?}", d);
    let mut i: Option<i32> = Some(1);
    *i.insert(7) += 1;
    println!("{:?}", i);
    println!("{:?}", Some(Some(3)).flatten());
    println!("{:?}", Some(Ok::<i32, String>(1)).transpose());
    println!("{:?}", Some(Err::<i32, String>(String::from("x"))).transpose());
    println!("{:?}", None::<Result<i32, String>>.transpose());
    let k = Some(5).inspect(|x| println!("saw {}", x));
    println!("{:?}", k);
    println!("{}", Some(2).expect("must"));
    // Result
    let r = parse("abc");
    let e = parse("bad");
    println!("{} {} {} {}", r.is_ok(), r.is_err(), e.is_ok(), e.is_err());
    println!("{} {}", r.clone().is_ok_and(|x| x == 3), e.clone().is_err_and(|s| s.len() > 3));
    println!("{:?} {:?} {:?} {:?}", r.clone().ok(), e.clone().ok(), r.clone().err(), e.clone().err());
    println!("{:?} {:?}", r.as_ref().map(|x| x + 1), e.as_ref().map_err(|s| s.len()));
    println!("{} {}", r.clone().map_or(0, |x| x * 2), e.clone().map_or(0, |x| x * 2));
    println!("{} {}", r.clone().map_or_else(|s| s.len() as i32, |x| x * 2), e.clone().map_or_else(|s| s.len() as i32, |x| x * 2));
    println!("{} {}", r.clone().map_or_default(|x| x * 2), e.clone().map_or_default(|x| x * 2));
    println!("{:?}", r.clone().and_then(|x| if x > 1 { Ok(x) } else { Err("small".to_string()) }));
    println!("{:?}", e.clone().or_else(|s| if s.len() > 1 { Ok::<i32, String>(0) } else { Err(s) }));
    println!("{:?} {:?}", r.clone().and(Ok::<char, String>('c')), e.clone().or(Ok::<i32, u8>(9)));
    println!("{} {} {}", r.clone().unwrap_or(0), e.clone().unwrap_or(0), e.clone().unwrap_or_else(|s| s.len() as i32));
    println!("{} {}", r.clone().unwrap_or_default(), e.clone().unwrap_or_default());
    println!("{} {}", r.clone().unwrap(), r.clone().expect("ok"));
    println!("{} {}", e.clone().unwrap_err(), e.clone().expect_err("err"));
    let mut rm: Result<i32, String> = Ok(1);
    if let Ok(x) = rm.as_mut() { *x += 1; }
    println!("{:?}", rm);
    let q = 5;
    let rr: Result<&i32, String> = Ok(&q);
    println!("{:?} {:?}", rr.clone().copied(), rr.cloned());
    println!("{:?}", Ok::<Option<i32>, String>(Some(1)).transpose());
    println!("{:?}", Ok::<Result<i32, String>, String>(Ok(1)).flatten());
    let _ = r.inspect(|x| println!("ok {}", x)).inspect_err(|e| println!("err {}", e));
    let _ = e.inspect(|x| println!("ok {}", x)).inspect_err(|e| println!("err {}", e));
}
