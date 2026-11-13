use std::rc::Rc;
fn main() {
    let r: Rc<[i32]> = Rc::from(vec![5, 3, 9]);
    for x in r.iter() { print!("{x} "); }
    for x in &*r { print!("{x}, "); }
    println!();
    println!("{:?} {} {} {:?} {:?} {:?}", r.to_vec(), r.contains(&3), r.is_empty(), r.first(), &r[1..], r.iter().max());
    let v: Vec<i32> = r.iter().map(|x| x * 2).collect();
    println!("{:?}", v);
}
