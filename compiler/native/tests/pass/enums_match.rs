// Enums with every variant shape, match with guards, or-patterns, and default binding modes.
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(i64),
    Rect { w: i64, h: i64 },
    Empty,
}

fn area(s: &Shape) -> i64 {
    match s {
        Shape::Circle(r) => 3 * r * r,
        Shape::Rect { w, h } if *w == *h => w * w,
        Shape::Rect { w, h } => w * h,
        Shape::Empty => 0,
    }
}

fn describe(n: i64) -> String {
    match n {
        0 => "zero".to_string(),
        1 | 2 | 3 => "small".to_string(),
        4..=9 => "medium".to_string(),
        _ => format!("big {}", n),
    }
}

fn main() {
    let shapes = vec![Shape::Circle(2), Shape::Rect { w: 3, h: 4 }, Shape::Rect { w: 5, h: 5 }, Shape::Empty];
    let mut total = 0;
    for s in &shapes {
        total += area(s);
        println!("{:?} -> {}", s, area(s));
    }
    println!("total {total}");
    for n in 3..5usize {
        println!("{}", describe(n as i64));
    }
    for n in 0..12 {
        print!("{} ", describe(n));
    }
    println!();
    let copy = shapes[1].clone();
    println!("{}", copy == shapes[1]);
    println!("{}", copy == shapes[2]);
    println!("{:?}", shapes);
}
