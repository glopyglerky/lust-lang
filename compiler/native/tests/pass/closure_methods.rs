// Method calls on captures pick the capture mode from the receiver.
#[derive(Debug)]
enum Shape {
    Circle(u32),
    Square(u32),
}
fn main() {
    let mut v: Vec<u32> = vec![1, 2];
    let len = || v.len();
    println!("{}", len());
    let mut push = |x: u32| v.push(x);
    push(3);
    push(4);
    println!("{:?}", v);
    let shape = Shape::Square(3);
    let area = || match shape {
        Shape::Circle(r) => r * r * 3,
        Shape::Square(s) => s * s,
    };
    println!("{} {:?}", area(), shape);
    let opt: Option<String> = Some(String::from("inner"));
    let peek = || match &opt {
        Some(s) => s.len(),
        None => 0,
    };
    println!("{} {:?}", peek(), opt);
}
