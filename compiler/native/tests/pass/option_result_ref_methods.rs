// Option::as_ref, as_mut, take, replace and Result::as_ref, as_mut,
// incl. a linked list emptied with take.
struct Node {
    val: i32,
    next: Option<Box<Node>>,
}
fn main() {
    let mut a: Option<String> = Some(String::from("x"));
    let r: Option<&String> = a.as_ref();
    println!("{:?}", r);
    if let Some(s) = a.as_mut() {
        s.push('y');
    }
    let old = a.replace(String::from("z"));
    println!("{:?} {:?}", old, a);
    let t = a.take();
    println!("{:?} {:?}", t, a);
    let mut list = Some(Box::new(Node { val: 1, next: Some(Box::new(Node { val: 2, next: None })) }));
    let mut sum = 0;
    while let Some(mut n) = list.take() {
        sum += n.val;
        list = n.next.take();
    }
    println!("{}", sum);
    let mut res: Result<i32, String> = Ok(3);
    if let Ok(v) = res.as_mut() {
        *v += 1;
    }
    println!("{:?} {:?}", res.as_ref(), res.as_ref().ok());
    let e: Result<i32, String> = Err(String::from("bad"));
    println!("{:?}", e.as_ref().err());
}
